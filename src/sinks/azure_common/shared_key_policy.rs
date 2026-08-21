use std::{collections::BTreeMap, fmt::Write as _, sync::Arc};

use async_trait::async_trait;
use azure_core::http::policies::{Policy, PolicyResult};
use azure_core::http::{Context, Request, Url};
use azure_core::{
    Result as AzureResult, base64,
    error::Error as AzureError,
    time::{OffsetDateTime, to_rfc7231},
};

use openssl::{hash::MessageDigest, pkey::PKey, sign::Signer};

/// Shared Key authorization policy for Azure Blob Storage requests.
///
/// This policy injects the required headers (x-ms-date, x-ms-version) if missing and
/// adds the `Authorization: SharedKey {account}:{signature}` header. The signature
/// is computed according to the "Authorize with Shared Key" rules for the Blob service:
///
/// StringToSign =
///   VERB + "\n" +
///   Content-Encoding + "\n" +
///   Content-Language + "\n" +
///   Content-Length + "\n" +
///   Content-MD5 + "\n" +
///   Content-Type + "\n" +
///   Date + "\n" +
///   If-Modified-Since + "\n" +
///   If-Match + "\n" +
///   If-None-Match + "\n" +
///   If-Unmodified-Since + "\n" +
///   Range + "\n" +
///   CanonicalizedHeaders +
///   CanonicalizedResource
///
/// Notes:
/// - We set x-ms-date, leaving the standard Date field empty in the signature.
/// - If Content-Length header is present with "0", the canonicalized value must be the empty string.
/// - Canonicalized headers include all x-ms-* headers (lowercased, sorted).
/// - Canonicalized resource is "/{account}{path}\n" + sorted lowercase query params,
///   parsed the way the service parses them (see `append_canonicalized_resource`).
///
#[derive(Debug)]
pub struct SharedKeyAuthorizationPolicy {
    account_name: String,
    account_key: Vec<u8>, // decoded from base64
    storage_version: String,
}

impl SharedKeyAuthorizationPolicy {
    /// Create a new shared key policy.
    ///
    /// - `account_name`: The storage account name.
    /// - `account_key_b64`: Base64-encoded storage account key.
    /// - `storage_version`: x-ms-version value to send (e.g. "2025-11-05").
    pub fn new(
        account_name: String,
        account_key_b64: String,
        storage_version: String,
    ) -> AzureResult<Self> {
        let account_key = base64::decode(account_key_b64.as_bytes()).map_err(|e| {
            AzureError::with_message(
                azure_core::error::ErrorKind::Other,
                format!("invalid account key base64: {e}"),
            )
        })?;
        Ok(Self {
            account_name,
            account_key,
            storage_version,
        })
    }

    fn ensure_ms_headers(&self, request: &mut Request) -> AzureResult<(String, String)> {
        // Always set x-ms-date and x-ms-version explicitly to known values for signing.
        let now = OffsetDateTime::now_utc();
        let ms_date = to_rfc7231(&now);
        request.insert_header("x-ms-date", ms_date.clone());
        let ms_version = self.storage_version.clone();
        request.insert_header("x-ms-version", ms_version.clone());
        Ok((ms_date, ms_version))
    }

    fn build_string_to_sign(
        &self,
        req: &Request,
        ms_date: &str,
        ms_version: &str,
    ) -> AzureResult<String> {
        let method = req.method().as_str();
        let url = req.url();

        let mut s = String::with_capacity(512);

        // VERB
        s.push_str(method);
        s.push('\n');

        // Resolve standard headers (case-insensitive) and write them in order required by the spec.
        // https://learn.microsoft.com/en-us/rest/api/storageservices/authorize-with-shared-key#shared-key-format-for-2009-09-19-and-later
        let header = |name: &str| -> Option<&str> {
            for (n, v) in req.headers().iter() {
                if n.as_str().eq_ignore_ascii_case(name) {
                    return Some(v.as_str());
                }
            }
            None
        };

        // Content-Encoding
        if let Some(v) = header("Content-Encoding") {
            s.push_str(v);
        }
        s.push('\n');

        // Content-Language
        if let Some(v) = header("Content-Language") {
            s.push_str(v);
        }
        s.push('\n');

        // Content-Length: some generated SDK operations (e.g. commit_block_list)
        // rely on the HTTP transport to add this header, so when it is absent the
        // body length must be signed instead of the empty string. Per the
        // 2015-02-21+ rules, a length of 0 is signed as the empty string.
        let content_length = match header("Content-Length") {
            Some(v) => v.parse::<u64>().ok(),
            None => Some(req.body().len() as u64),
        };
        match content_length {
            Some(0) => {}
            Some(n) => {
                let _ = write!(s, "{}", n);
            }
            // Unparseable header value: sign it verbatim so wire and signature agree.
            None => {
                if let Some(v) = header("Content-Length") {
                    s.push_str(v);
                }
            }
        }
        s.push('\n');

        // Content-MD5
        if let Some(v) = header("Content-MD5") {
            s.push_str(v);
        }
        s.push('\n');

        // Content-Type
        if let Some(v) = header("Content-Type") {
            s.push_str(v);
        }
        s.push('\n');

        // Date (unused when x-ms-date is used)
        s.push('\n');

        // If-Modified-Since
        if let Some(v) = header("If-Modified-Since") {
            s.push_str(v);
        }
        s.push('\n');

        // If-Match
        if let Some(v) = header("If-Match") {
            s.push_str(v);
        }
        s.push('\n');

        // If-None-Match
        if let Some(v) = header("If-None-Match") {
            s.push_str(v);
        }
        s.push('\n');

        // If-Unmodified-Since
        if let Some(v) = header("If-Unmodified-Since") {
            s.push_str(v);
        }
        s.push('\n');

        // Range
        if let Some(v) = header("Range") {
            s.push_str(v);
        }
        s.push('\n');

        // CanonicalizedHeaders: include all x-ms-* headers, lowercased, sorted by name.
        // If multiple values for the same header exist, sort values and join with commas.
        let mut xms: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (name, value) in req.headers().iter() {
            let key = name.as_str().to_ascii_lowercase();
            if key.starts_with("x-ms-") {
                xms.entry(key)
                    .or_default()
                    .push(value.as_str().trim().to_string());
            }
        }
        // Ensure required headers are present (they should have been inserted).
        xms.entry("x-ms-date".to_string())
            .or_default()
            .push(ms_date.to_string());
        xms.entry("x-ms-version".to_string())
            .or_default()
            .push(ms_version.to_string());

        for (k, mut vals) in xms {
            vals.sort();
            vals.dedup();
            let joined = vals.join(",");
            let _ = writeln!(s, "{}:{}", k, joined);
        }

        // CanonicalizedResource
        append_canonicalized_resource(&mut s, &self.account_name, url)?;

        Ok(s)
    }

    fn sign(&self, string_to_sign: &str) -> AzureResult<String> {
        let pkey = PKey::hmac(&self.account_key).map_err(|e| {
            AzureError::with_message(
                azure_core::error::ErrorKind::Other,
                format!("failed to create HMAC key: {e}"),
            )
        })?;
        let mut signer = Signer::new(MessageDigest::sha256(), &pkey).map_err(|e| {
            AzureError::with_message(
                azure_core::error::ErrorKind::Other,
                format!("failed to create signer: {e}"),
            )
        })?;
        signer.update(string_to_sign.as_bytes()).map_err(|e| {
            AzureError::with_message(
                azure_core::error::ErrorKind::Other,
                format!("signer update failed: {e}"),
            )
        })?;
        let mac = signer.sign_to_vec().map_err(|e| {
            AzureError::with_message(
                azure_core::error::ErrorKind::Other,
                format!("signer sign failed: {e}"),
            )
        })?;
        Ok(base64::encode(&mac))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Policy for SharedKeyAuthorizationPolicy {
    async fn send(
        &self,
        ctx: &Context,
        request: &mut Request,
        next: &[Arc<dyn Policy>],
    ) -> PolicyResult {
        // Ensure required x-ms headers are present
        let (ms_date, ms_version) = self.ensure_ms_headers(request)?;
        // Build string to sign
        let sts = self.build_string_to_sign(request, &ms_date, &ms_version)?;
        let signature = self.sign(&sts)?;

        // Authorization: SharedKey {account}:{signature}
        request.insert_header(
            "authorization",
            format!("SharedKey {}:{}", self.account_name, signature),
        );

        // Continue pipeline
        next[0].send(ctx, request, &next[1..]).await
    }
}

// ---------- Helpers ----------

fn append_canonicalized_resource(s: &mut String, account: &str, url: &Url) -> AzureResult<()> {
    // "/{account_name}{path}\n"
    s.push('/');
    s.push_str(account);
    // Append the URL path exactly as-is (per spec).
    s.push_str(url.path());

    // Canonicalized query: lowercase names, sort by name, join multi-values by comma, each line "name:value\n"
    // https://learn.microsoft.com/en-us/rest/api/storageservices/authorize-with-shared-key#shared-key-format-for-2009-09-19-and-later
    //
    // MEZMO: the service parses the query string like .NET's
    // HttpUtility.ParseQueryString and signs every parameter it finds, so:
    // - an explicit empty value is kept: `prefix=` signs as `prefix:`
    // - a token without `=` is stored under an empty name with the token as
    //   its value: the SDK's list request appends `&flat`, which signs as `:flat`
    // - an empty token (`&&` or a trailing `&`) is an empty value under the
    //   empty name and signs as `:`, but an entirely empty query (a bare `?`)
    //   contributes nothing
    // Dropping any of them produces a 403 from the real service, while Azurite
    // ignores all of them and rejects a signature that includes them. That is
    // why `QueryNormalizationPolicy` removes such tokens before signing; this
    // function stays faithful to the service regardless.
    if let Some(query) = url.query().filter(|query| !query.is_empty()) {
        let mut qp_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for token in query.split('&') {
            let (name, value) = canonical_query_pair(token);
            qp_map
                .entry(name.to_ascii_lowercase())
                .or_default()
                .push(value);
        }
        for (k, mut vals) in qp_map {
            vals.sort();
            let _ = write!(s, "\n{}:{}", k, vals.join(","));
        }
    }

    Ok(())
}

/// Splits one raw `application/x-www-form-urlencoded` query token into the
/// decoded (name, value) pair the service uses for signing.
fn canonical_query_pair(token: &str) -> (String, String) {
    let (name, value) = url::form_urlencoded::parse(token.as_bytes())
        .next()
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .unwrap_or_default();
    if token.contains('=') {
        (name, value)
    } else {
        (String::new(), name)
    }
}

#[cfg(test)]
mod tests {
    use azure_core::http::Method;

    use super::*;

    fn canonicalized_resource(url: &str) -> String {
        let url = Url::parse(url).unwrap();
        let mut s = String::new();
        append_canonicalized_resource(&mut s, "account", &url).unwrap();
        s
    }

    #[test]
    fn canonicalized_resource_without_query() {
        assert_eq!(
            canonicalized_resource(
                "https://account.blob.core.windows.net/container/dir%2Ffile.log"
            ),
            "/account/container/dir%2Ffile.log"
        );
        // Verified against the service: a bare `?` must not sign a `:` line.
        assert_eq!(
            canonicalized_resource("https://account.blob.core.windows.net/container?"),
            "/account/container"
        );
    }

    #[test]
    fn canonicalized_resource_signs_key_only_and_empty_valued_params() {
        // Mirrors the string the service echoed back for the SDK's list request:
        // `?comp=list&flat&restype=container&include=tags&prefix=`
        assert_eq!(
            canonicalized_resource(
                "https://account.blob.core.windows.net/container?comp=list&flat&restype=container&include=tags&prefix="
            ),
            "/account/container\n:flat\ncomp:list\ninclude:tags\nprefix:\nrestype:container"
        );
    }

    #[test]
    fn canonicalized_resource_signs_empty_tokens_as_nameless_empty_values() {
        // Verified against the service: a listing with `&&` or a trailing `&`
        // is accepted only when the canonicalized resource carries the `:` line.
        assert_eq!(
            canonicalized_resource(
                "https://account.blob.core.windows.net/container?comp=list&&restype=container&maxresults=1"
            ),
            "/account/container\n:\ncomp:list\nmaxresults:1\nrestype:container"
        );
        assert_eq!(
            canonicalized_resource(
                "https://account.blob.core.windows.net/container?comp=list&restype=container&"
            ),
            "/account/container\n:\ncomp:list\nrestype:container"
        );
    }

    #[test]
    fn canonicalized_resource_decodes_lowercases_and_sorts() {
        assert_eq!(
            canonicalized_resource(
                "https://account.blob.core.windows.net/container?Prefix=pw-consolidation+123%2F&comp=list&include=tags&include=metadata"
            ),
            "/account/container\ncomp:list\ninclude:metadata,tags\nprefix:pw-consolidation 123/"
        );
    }

    #[test]
    fn string_to_sign_for_list_request_matches_service() {
        let policy = SharedKeyAuthorizationPolicy::new(
            "account".to_owned(),
            "ZmFrZS10ZXN0LWFjY291bnQta2V5".to_owned(),
            "2025-11-05".to_owned(),
        )
        .unwrap();
        let url = Url::parse(
            "https://account.blob.core.windows.net/container?comp=list&flat&restype=container",
        )
        .unwrap();
        let mut request = Request::new(url, Method::Get);
        request.insert_header("accept", "application/xml");
        request.insert_header("content-type", "application/xml");
        request.insert_header("x-ms-client-request-id", "req-id");

        let sts = policy
            .build_string_to_sign(&request, "Fri, 21 Aug 2026 12:59:27 GMT", "2025-11-05")
            .unwrap();

        assert_eq!(
            sts,
            "GET\n\n\n\n\napplication/xml\n\n\n\n\n\n\n\
             x-ms-client-request-id:req-id\n\
             x-ms-date:Fri, 21 Aug 2026 12:59:27 GMT\n\
             x-ms-version:2025-11-05\n\
             /account/container\n:flat\ncomp:list\nrestype:container"
        );
    }
}
