//! SSRF guard for HTTP clients that dial user-supplied endpoints.
//!
//! The endpoint of a component such as the `http_client` source is attacker-controlled:
//! a tenant can point it at any hostname. Validating that hostname when the config is
//! saved is not enough, because DNS is re-resolved on every connection and the answer
//! can change between the check and the dial (DNS rebinding). A check that is not bound
//! to the connection is advisory.
//!
//! This module moves the check to the point where the name is actually resolved, so what
//! gets validated is what gets connected to.
//!
//! Note that hyper's `HttpConnector` only consults the resolver for hostnames: a URI that
//! is already an IP literal is parsed and dialed directly. Literals are therefore checked
//! separately, where the request is issued (see `HttpClient::send`). That check is not
//! subject to rebinding, since no name resolution is involved.

use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    task::{Context, Poll},
};

use futures::{FutureExt, future::BoxFuture};
use hyper::client::connect::dns::{GaiResolver, Name};
use tower::Service;
use tracing::warn;

/// Returns `true` if `addr` is in a range a user-supplied endpoint must never reach:
/// loopback, link-local (the cloud metadata service lives at `169.254.169.254`), private
/// networks, and the various reserved or special-purpose blocks.
///
/// This set is fixed by RFC. It is deliberately not configurable.
pub fn is_blocked(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(addr) => is_blocked_v4(addr),
        IpAddr::V6(addr) => is_blocked_v6(addr),
    }
}

fn is_blocked_v4(addr: Ipv4Addr) -> bool {
    let [a, b, c, _] = addr.octets();
    match a {
        // "This network" (RFC 1122)
        0 => true,
        // Private (RFC 1918)
        10 => true,
        // Shared address space / CGNAT (RFC 6598)
        100 => (64..=127).contains(&b),
        // Loopback (RFC 1122)
        127 => true,
        // Link-local, which includes the cloud metadata service (RFC 3927)
        169 => b == 254,
        // Private (RFC 1918)
        172 => (16..=31).contains(&b),
        192 => match (b, c) {
            // IETF protocol assignments (RFC 6890)
            (0, 0) => true,
            // TEST-NET-1 (RFC 5737)
            (0, 2) => true,
            // 6to4 relay anycast (RFC 3068)
            (88, 99) => true,
            // Private (RFC 1918)
            (168, _) => true,
            _ => false,
        },
        198 => match (b, c) {
            // Benchmarking (RFC 2544)
            (18 | 19, _) => true,
            // TEST-NET-2 (RFC 5737)
            (51, 100) => true,
            _ => false,
        },
        // TEST-NET-3 (RFC 5737)
        203 => b == 0 && c == 113,
        // Multicast (RFC 5771). Also covers 233.252.0.0/24 (MCAST-TEST-NET).
        224..=239 => true,
        // Reserved (RFC 1112), including the broadcast address
        240..=255 => true,
        _ => false,
    }
}

fn is_blocked_v6(addr: Ipv6Addr) -> bool {
    // These must be judged before unwrapping to IPv4: `::1` and `::` are themselves
    // IPv4-compatible addresses and would otherwise unwrap to 0.0.0.1 and 0.0.0.0.
    if addr.is_loopback() || addr.is_unspecified() || addr.is_multicast() {
        return true;
    }

    // IPv4-mapped (`::ffff:a.b.c.d`) and the deprecated IPv4-compatible (`::a.b.c.d`)
    // forms both reach an IPv4 destination, so they are judged as IPv4. Without this,
    // `::ffff:169.254.169.254` walks straight past the v6 checks.
    if let Some(v4) = addr.to_ipv4() {
        return is_blocked_v4(v4);
    }

    let segments = addr.segments();

    // The NAT64 well-known prefix (RFC 6052) embeds the IPv4 destination.
    if segments[0] == 0x0064 && segments[1] == 0xff9b && segments[2..6] == [0, 0, 0, 0] {
        let [a, b] = segments[6].to_be_bytes();
        let [c, d] = segments[7].to_be_bytes();
        return is_blocked_v4(Ipv4Addr::new(a, b, c, d));
    }

    let octets = addr.octets();

    // Unique local (RFC 4193)
    (octets[0] & 0xfe) == 0xfc
        // Link-local unicast (RFC 4291)
        || (octets[0] == 0xfe && (octets[1] & 0xc0) == 0x80)
        // Site-local (RFC 3513). Deprecated by RFC 3879, but a deprecated prefix is
        // still a routable one: hosts and routers that predate the deprecation still
        // answer on it, so it stays reachable from the worker and must be rejected.
        || (octets[0] == 0xfe && (octets[1] & 0xc0) == 0xc0)
        // Documentation (RFC 3849)
        || (segments[0] == 0x2001 && segments[1] == 0x0db8)
        // Discard-only (RFC 6666)
        || (segments[0] == 0x0100 && segments[1..4] == [0, 0, 0])
}

/// Wraps hyper's system resolver and drops any address that a user-supplied endpoint is
/// not permitted to reach.
///
/// The guard is a runtime flag rather than a separate type so that guarded and unguarded
/// clients share a single connector type, which keeps `HttpClient` concrete and leaves
/// every existing caller untouched.
#[derive(Clone)]
pub struct GuardedResolver {
    inner: GaiResolver,
    enabled: bool,
}

impl GuardedResolver {
    /// Creates a resolver. When `enabled` is false this defers to the system resolver
    /// and filters nothing.
    pub fn new(enabled: bool) -> Self {
        Self {
            inner: GaiResolver::new(),
            enabled,
        }
    }
}

impl Service<Name> for GuardedResolver {
    type Response = std::vec::IntoIter<SocketAddr>;
    type Error = io::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, name: Name) -> Self::Future {
        let enabled = self.enabled;
        let host = name.as_str().to_owned();
        let lookup = self.inner.call(name);

        async move {
            let addrs = lookup.await?;

            if !enabled {
                return Ok(addrs.collect::<Vec<_>>().into_iter());
            }

            let (permitted, blocked): (Vec<_>, Vec<_>) =
                addrs.partition(|addr| !is_blocked(addr.ip()));

            if !blocked.is_empty() {
                warn!(
                    message = "Blocked a resolved address in a restricted range for a user-supplied endpoint.",
                    host = %host,
                    blocked = ?blocked.iter().map(|addr| addr.ip()).collect::<Vec<_>>(),
                );
            }

            if permitted.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("{host} resolves only to addresses in restricted ranges"),
                ));
            }

            Ok(permitted.into_iter())
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocked(addr: &str) -> bool {
        is_blocked(addr.parse::<IpAddr>().expect("valid address"))
    }

    #[test]
    fn blocks_cloud_metadata() {
        // The address the VM-713 report read IAM credentials from.
        assert!(blocked("169.254.169.254"));
        assert!(blocked("169.254.0.1"));
    }

    #[test]
    fn blocks_loopback() {
        // The Vector GraphQL control-plane leg of the same report.
        assert!(blocked("127.0.0.1"));
        assert!(blocked("127.1.2.3"));
        assert!(blocked("::1"));
    }

    #[test]
    fn blocks_private_ranges() {
        assert!(blocked("10.0.0.1"));
        assert!(blocked("172.16.0.1"));
        assert!(blocked("172.31.255.255"));
        assert!(blocked("192.168.1.1"));
        assert!(blocked("100.64.0.1"));
    }

    #[test]
    fn allows_public_addresses() {
        assert!(!blocked("1.1.1.1"));
        assert!(!blocked("8.8.8.8"));
        assert!(!blocked("93.184.216.34"));
        // Adjacent to private ranges but outside them.
        assert!(!blocked("172.15.255.255"));
        assert!(!blocked("172.32.0.0"));
        assert!(!blocked("100.63.255.255"));
        assert!(!blocked("100.128.0.0"));
        assert!(!blocked("169.253.255.255"));
        assert!(!blocked("2606:4700:4700::1111"));
    }

    #[test]
    fn blocks_ipv4_mapped_and_compatible_forms() {
        // An IPv4-mapped address reaches the same host, so it must be judged as IPv4.
        assert!(blocked("::ffff:169.254.169.254"));
        assert!(blocked("::ffff:127.0.0.1"));
        assert!(blocked("::ffff:10.0.0.1"));
        assert!(!blocked("::ffff:1.1.1.1"));
    }

    #[test]
    fn blocks_nat64_embedded_targets() {
        // 64:ff9b::169.254.169.254
        assert!(blocked("64:ff9b::a9fe:a9fe"));
        // 64:ff9b::127.0.0.1
        assert!(blocked("64:ff9b::7f00:1"));
        // 64:ff9b::1.1.1.1 embeds a public target.
        assert!(!blocked("64:ff9b::101:101"));
    }

    #[test]
    fn blocks_ipv6_internal_ranges() {
        assert!(blocked("::"));
        assert!(blocked("fc00::1"));
        assert!(blocked("fd12:3456::1"));
        assert!(blocked("fe80::1"));
        assert!(blocked("febf:ffff::1"));
        // Site-local (fec0::/10): deprecated, still routable.
        assert!(blocked("fec0::1"));
        assert!(blocked("feff:ffff::1"));
        assert!(blocked("ff02::1"));
        assert!(blocked("2001:db8::1"));
        assert!(blocked("100::1"));
    }

    #[test]
    fn blocks_reserved_and_special_ranges() {
        assert!(blocked("0.0.0.0"));
        assert!(blocked("0.1.2.3"));
        assert!(blocked("192.0.0.1"));
        assert!(blocked("192.0.2.1"));
        assert!(blocked("192.88.99.1"));
        assert!(blocked("198.18.0.1"));
        assert!(blocked("198.19.255.255"));
        assert!(blocked("198.51.100.1"));
        assert!(blocked("203.0.113.1"));
        assert!(blocked("224.0.0.1"));
        assert!(blocked("233.252.0.1"));
        assert!(blocked("240.0.0.1"));
        assert!(blocked("255.255.255.255"));
    }

    #[tokio::test]
    async fn resolver_rejects_internal_only_name() {
        let mut resolver = GuardedResolver::new(true);
        let name: Name = "localhost".parse().expect("valid name");

        let err = resolver
            .call(name)
            .await
            .expect_err("localhost resolves only to loopback and must be rejected");

        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
    }

    #[tokio::test]
    async fn disabled_resolver_permits_internal_name() {
        let mut resolver = GuardedResolver::new(false);
        let name: Name = "localhost".parse().expect("valid name");

        let addrs = resolver
            .call(name)
            .await
            .expect("guard is disabled, so loopback must resolve");

        assert!(addrs.count() > 0);
    }
}
