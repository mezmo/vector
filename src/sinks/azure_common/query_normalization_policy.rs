use std::sync::Arc;

use async_trait::async_trait;
use azure_core::http::policies::{Policy, PolicyResult};
use azure_core::http::{Context, Request, Url};

/// Removes query tokens that carry no meaning for the Blob service before a
/// request is signed and sent.
///
/// Every Blob REST query parameter has the form `name=value`. Anything else
/// in the query string is nameless: a bare token such as the `flat` the 0.7
/// SDK's list operation appends (a code generation artifact that later SDK
/// releases dropped), or the empty token left by `&&` or a trailing `&`. The
/// real service stores such tokens under an empty name and includes them in
/// the Shared Key signature (`:flat`, `:`), while Azurite discards them, so no
/// single signature satisfies both. Stripping them keeps the request on the
/// documented wire format and makes the signature unambiguous.
///
/// This must run before `SharedKeyAuthorizationPolicy`.
#[derive(Debug, Default)]
pub struct QueryNormalizationPolicy;

pub fn strip_nameless_query_tokens(url: &mut Url) {
    let Some(query) = url.query() else {
        return;
    };
    if query.split('&').all(|token| token.contains('=')) {
        return;
    }

    let normalized = query
        .split('&')
        .filter(|token| token.contains('='))
        .collect::<Vec<_>>()
        .join("&");
    url.set_query((!normalized.is_empty()).then_some(normalized.as_str()));
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Policy for QueryNormalizationPolicy {
    async fn send(
        &self,
        ctx: &Context,
        request: &mut Request,
        next: &[Arc<dyn Policy>],
    ) -> PolicyResult {
        strip_nameless_query_tokens(request.url_mut());
        next[0].send(ctx, request, &next[1..]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalized(url: &str) -> String {
        let mut url = Url::parse(url).unwrap();
        strip_nameless_query_tokens(&mut url);
        url.to_string()
    }

    #[test]
    fn strips_the_sdk_list_flat_token() {
        assert_eq!(
            normalized(
                "https://account.blob.core.windows.net/container?comp=list&flat&restype=container&prefix=a+b%2F"
            ),
            "https://account.blob.core.windows.net/container?comp=list&restype=container&prefix=a+b%2F"
        );
    }

    #[test]
    fn strips_empty_tokens() {
        // The service signs each empty token as a nameless empty value, so a
        // double or trailing ampersand is as ambiguous as a bare token.
        assert_eq!(
            normalized(
                "https://account.blob.core.windows.net/container?comp=list&&restype=container&"
            ),
            "https://account.blob.core.windows.net/container?comp=list&restype=container"
        );
    }

    #[test]
    fn leaves_well_formed_queries_untouched() {
        let url = "https://account.blob.core.windows.net/container/blob?comp=tags&sig=abc%2Fdef%3D";
        assert_eq!(normalized(url), url);
        let url = "https://account.blob.core.windows.net/container/blob?prefix=";
        assert_eq!(normalized(url), url);
        let url = "https://account.blob.core.windows.net/container/blob";
        assert_eq!(normalized(url), url);
    }

    #[test]
    fn removes_the_query_entirely_when_only_nameless_tokens_remain() {
        for url in [
            "https://account.blob.core.windows.net/container?flat",
            "https://account.blob.core.windows.net/container?&",
            "https://account.blob.core.windows.net/container?",
        ] {
            assert_eq!(
                normalized(url),
                "https://account.blob.core.windows.net/container"
            );
        }
    }
}
