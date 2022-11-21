use std::sync::{Arc, RwLock};
use std::str::FromStr;
use std::time::Duration;

pub use goauth::scopes::Scope;
use goauth::{
    auth::{JwtClaims, Token, TokenErr},
    credentials::Credentials,
    GoErr,
};
use http::{uri::PathAndQuery, Uri};
use hyper::header::AUTHORIZATION;
use once_cell::sync::Lazy;
use smpl_jwt::Jwt;
use snafu::{ResultExt, Snafu};
use tokio::{sync::watch, time::Instant};
use vector_config::configurable_component;

use crate::{config::ProxyConfig, http::HttpClient, http::HttpError};

const SERVICE_ACCOUNT_TOKEN_URL: &str =
    "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";

pub const PUBSUB_URL: &str = "https://pubsub.googleapis.com";

pub static PUBSUB_ADDRESS: Lazy<String> = Lazy::new(|| {
    std::env::var("EMULATOR_ADDRESS").unwrap_or_else(|_| "http://localhost:8681".into())
});

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum GcpError {
    #[snafu(display("This requires one of api_key or credentials_path to be defined"))]
    MissingAuth,
    #[snafu(display("Invalid GCP credentials: {}", source))]
    InvalidCredentials { source: GoErr },
    #[snafu(display("Invalid GCP API key: {}", source))]
    InvalidApiKey { source: base64::DecodeError },
    #[snafu(display("Healthcheck endpoint forbidden"))]
    HealthcheckForbidden,
    #[snafu(display("Invalid RSA key in GCP credentials: {}", source))]
    InvalidRsaKey { source: GoErr },
    #[snafu(display("Failed to get OAuth token: {}", source))]
    GetToken { source: GoErr },
    #[snafu(display("Failed to get OAuth token text: {}", source))]
    GetTokenBytes { source: hyper::Error },
    #[snafu(display("Failed to get implicit GCP token: {}", source))]
    GetImplicitToken { source: HttpError },
    #[snafu(display("Failed to parse OAuth token JSON: {}", source))]
    TokenFromJson { source: TokenErr },
    #[snafu(display("Failed to parse OAuth token JSON text: {}", source))]
    TokenJsonFromStr { source: serde_json::Error },
    #[snafu(display("Failed to build HTTP client: {}", source))]
    BuildHttpClient { source: HttpError },
}

/// Configuration of the authentication strategy for interacting with GCP services.
// TODO: We're duplicating the "either this or that" verbiage for each field because this struct gets flattened into the
// component config types, which means all that's carried over are the fields, not the type itself.
//
// Seems like we really really have it as a nested field -- i.e. `auth.api_key` -- which is a closer fit to how we do
// similar things in configuration (TLS, framing, decoding, etc.). Doing so would let us embed the type itself, and
// hoist up the common documentation bits to the docs for the type rather than the fields.
#[configurable_component]
#[derive(Clone, Debug, Default)]
pub struct GcpAuthConfig {
    /// An API key. ([documentation](https://cloud.google.com/docs/authentication/api-keys))
    ///
    /// Either an API key or JSON credentials (as a string, or a file path) can be specified.
    ///
    /// If all are unset, Vector checks the `GOOGLE_APPLICATION_CREDENTIALS` environment variable for a filename. If no
    /// filename is named, Vector will attempt to fetch an instance service account for the compute instance the program is
    /// running on. If Vector is not running on a GCE instance, then you must define eith an API key or service account
    /// credentials JSON file.
    pub api_key: Option<String>,

    /// Path to a service account credentials JSON file. ([documentation](https://cloud.google.com/docs/authentication/production#manually))
    ///
    /// Either an API key or JSON credentials (as a string, or a file path) can be specified.
    ///
    /// If all are unset, Vector checks the `GOOGLE_APPLICATION_CREDENTIALS` environment variable for a filename. If no
    /// filename is named, Vector will attempt to fetch an instance service account for the compute instance the program is
    /// running on. If Vector is not running on a GCE instance, then you must define eith an API key or service account
    /// credentials JSON file.
    pub credentials_path: Option<String>,

    /// JSON Credentials as a string. ([documentation](https://cloud.google.com/docs/authentication/production#manually))
    ///
    /// Either an API key or JSON credentials (as a string, or a file path) can be specified.
    ///
    /// If all are unset, Vector checks the `GOOGLE_APPLICATION_CREDENTIALS` environment variable for a filename. If no
    /// filename is named, Vector will attempt to fetch an instance service account for the compute instance the program is
    /// running on. If Vector is not running on a GCE instance, then you must define eith an API key or service account
    /// credentials JSON file.
    pub credentials_json: Option<String>,

    /// Skip all authentication handling. For use with integration tests only.
    #[serde(default, skip_serializing)]
    pub skip_authentication: bool,
}

impl GcpAuthConfig {
    pub async fn build(&self, scope: Scope) -> crate::Result<GcpAuthenticator> {
        Ok(if self.skip_authentication {
            GcpAuthenticator::None
        } else {
            let creds_path = self.credentials_path.as_ref();
            match (&creds_path, &self.credentials_json, &self.api_key) {
                (Some(path), _, _) => GcpAuthenticator::from_file(path, scope).await?,
                (None, Some(credentials_json), _) => GcpAuthenticator::from_str(credentials_json, scope).await?,
                (None, None, Some(api_key)) => GcpAuthenticator::from_api_key(api_key)?,
                (None, None, None) => GcpAuthenticator::None,
            }
        })
    }
}

#[derive(Clone, Debug)]
pub enum GcpAuthenticator {
    Credentials(Arc<InnerCreds>),
    ApiKey(Box<str>),
    None,
}

#[derive(Debug)]
pub struct InnerCreds {
    creds: Option<(Credentials, Scope)>,
    token: RwLock<Token>,
}

impl GcpAuthenticator {
    async fn from_file(path: &str, scope: Scope) -> crate::Result<Self> {
        let creds = Credentials::from_file(path).context(InvalidCredentialsSnafu)?;
        let token = RwLock::new(fetch_token(&creds, &scope).await?);
        let creds = Some((creds, scope));
        Ok(Self::Credentials(Arc::new(InnerCreds { creds, token })))
    }

    async fn from_str(json_str: &str, scope: Scope) -> crate::Result<Self> {
        let creds = Credentials::from_str(json_str).context(InvalidCredentialsSnafu)?;
        let token = RwLock::new(fetch_token(&creds, &scope).await?);
        let creds = Some((creds, scope));
        Ok(Self::Credentials(Arc::new(InnerCreds { creds, token })))
    }

    fn from_api_key(api_key: &str) -> crate::Result<Self> {
        base64::decode_config(api_key, base64::URL_SAFE).context(InvalidApiKeySnafu)?;
        Ok(Self::ApiKey(api_key.into()))
    }

    pub fn make_token(&self) -> Option<String> {
        match self {
            Self::Credentials(inner) => Some(inner.make_token()),
            Self::ApiKey(_) | Self::None => None,
        }
    }

    pub fn apply<T>(&self, request: &mut http::Request<T>) {
        if let Some(token) = self.make_token() {
            request
                .headers_mut()
                .insert(AUTHORIZATION, token.parse().unwrap());
        }
        self.apply_uri(request.uri_mut());
    }

    pub fn apply_uri(&self, uri: &mut Uri) {
        match self {
            Self::Credentials(_) | Self::None => (),
            Self::ApiKey(api_key) => {
                let mut parts = uri.clone().into_parts();
                let path = parts
                    .path_and_query
                    .as_ref()
                    .map_or("/", PathAndQuery::path);
                let paq = format!("{path}?key={api_key}");
                // The API key is verified above to only contain
                // URL-safe characters. That key is added to a path
                // that came from a successfully parsed URI. As such,
                // re-parsing the string cannot fail.
                parts.path_and_query =
                    Some(paq.parse().expect("Could not re-parse path and query"));
                *uri = Uri::from_parts(parts).expect("Could not re-parse URL");
            }
        }
    }

    pub fn spawn_regenerate_token(&self) -> watch::Receiver<()> {
        let (sender, receiver) = watch::channel(());
        tokio::spawn(self.clone().token_regenerator(sender));
        receiver
    }

    async fn token_regenerator(self, sender: watch::Sender<()>) {
        match self {
            Self::Credentials(inner) => {
                let period =
                    Duration::from_secs(inner.token.read().unwrap().expires_in() as u64 / 2);
                let mut interval = tokio::time::interval_at(Instant::now() + period, period);
                loop {
                    interval.tick().await;
                    debug!("Renewing GCP authentication token.");
                    match inner.regenerate_token().await {
                        Ok(()) => sender.send_replace(()),
                        Err(error) => {
                            error!(
                                message = "Failed to update GCP authentication token.",
                                %error
                            );
                        }
                    }
                }
            }
            Self::ApiKey(_) | Self::None => {
                // This keeps the sender end of the watch open without
                // actually sending anything, effectively creating an
                // empty watch stream.
                sender.closed().await
            }
        }
    }
}

impl InnerCreds {
    async fn regenerate_token(&self) -> crate::Result<()> {
        let token = match &self.creds {
            Some((creds, scope)) => fetch_token(creds, scope).await?,
            None => get_token_implicit().await?,
        };
        *self.token.write().unwrap() = token;
        Ok(())
    }

    fn make_token(&self) -> String {
        let token = self.token.read().unwrap();
        format!("{} {}", token.token_type(), token.access_token())
    }
}

async fn fetch_token(creds: &Credentials, scope: &Scope) -> crate::Result<Token> {
    let claims = JwtClaims::new(creds.iss(), scope, creds.token_uri(), None, None);
    let rsa_key = creds.rsa_key().context(InvalidRsaKeySnafu)?;
    let jwt = Jwt::new(claims, rsa_key, None);

    debug!(
        message = "Fetching GCP authentication token.",
        project = ?creds.project(),
        iss = ?creds.iss(),
        token_uri = ?creds.token_uri(),
    );
    goauth::get_token(&jwt, creds)
        .await
        .context(GetTokenSnafu)
        .map_err(Into::into)
}

async fn get_token_implicit() -> Result<Token, GcpError> {
    debug!("Fetching implicit GCP authentication token.");
    let req = http::Request::get(SERVICE_ACCOUNT_TOKEN_URL)
        .header("Metadata-Flavor", "Google")
        .body(hyper::Body::empty())
        .unwrap();

    let proxy = ProxyConfig::from_env();
    let res = HttpClient::new(None, &proxy)
        .context(BuildHttpClientSnafu)?
        .send(req)
        .await
        .context(GetImplicitTokenSnafu)?;

    let body = res.into_body();
    let bytes = hyper::body::to_bytes(body)
        .await
        .context(GetTokenBytesSnafu)?;

    // Token::from_str is irresponsible and may panic!
    match serde_json::from_slice::<Token>(&bytes) {
        Ok(token) => Ok(token),
        Err(error) => Err(match serde_json::from_slice::<TokenErr>(&bytes) {
            Ok(error) => GcpError::TokenFromJson { source: error },
            Err(_) => GcpError::TokenJsonFromStr { source: error },
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_downcast_matches;

    #[tokio::test]
    async fn fails_missing_creds() {
        let error = build_auth("").await.expect_err("build failed to error");
        assert_downcast_matches!(error, GcpError, GcpError::GetImplicitToken { .. });
        // This should be a more relevant error
    }

    #[tokio::test]
    async fn skip_authentication() {
        let auth = build_auth(
            r#"
                skip_authentication = true
                api_key = "testing"
            "#,
        )
        .await
        .expect("build_auth failed");
        assert!(matches!(auth, GcpAuthenticator::None));
    }

    #[tokio::test]
    async fn uses_api_key() {
        let key = crate::test_util::random_string(16);

        let auth = build_auth(&format!(r#"api_key = "{key}""#))
            .await
            .expect("build_auth failed");
        assert!(matches!(auth, GcpAuthenticator::ApiKey(..)));

        assert_eq!(
            apply_uri(&auth, "http://example.com"),
            format!("http://example.com/?key={key}")
        );
        assert_eq!(
            apply_uri(&auth, "http://example.com/"),
            format!("http://example.com/?key={key}")
        );
        assert_eq!(
            apply_uri(&auth, "http://example.com/path"),
            format!("http://example.com/path?key={key}")
        );
        assert_eq!(
            apply_uri(&auth, "http://example.com/path1/"),
            format!("http://example.com/path1/?key={key}")
        );
    }

    #[tokio::test]
    async fn uses_credentials_json() {
        let auth = GcpAuthenticator::from_str(r#"
        {
            "type": "service_account",
            "project_id": "essential-topic-368917",
            "private_key_id": "ca1b71ee693c444c8e6e641470e8bc35d1411964",
            "private_key": "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQDcZAeDGq3NC8hV\nLMdpSNhMOdynPTPn1bdCui9IeqPNXM0JxHDJB7/GhpyjiJPjp2QK2sd+cC7oxrYS\nHrpJ3dJwhhMrI+cCQ3QPBHEPRdoG1T7P5twPl0zbOAbaBqAmds4kfykUoAMumb3n\nJ1RAZ25FXeRYYCYZ+dKdSo/Dha1eHgEZBylRWUqV2L6aG4wmoMcp4EcxR8CdWm3z\nB/lBVzFUle0fgkceT0OrCBrFvllkY1aecFWMGp1cdb0ooX+i5n1dtnLxM6X1dTcE\nstKCTGZ7c8SaZuucRGQr5e4chcT96ZLBz6C4ZelsgXPhdCNEe1Ntq+DglCi2cIbx\nr/f7bYPVAgMBAAECggEANbTQmdPOB7o30v3LCG6eexDcowqIlBXiB0o8zIJKWXik\nZJ1wyKRxSO0zzawyMddwSy7eT4MCA2qtIsRHLEn4hsA9epVQrZ4HccNo08p3Y5Pi\nryI4fTonGgLQtJ/JtiXcfUtZlPObYudHPkW4w8sQtam4RAsGLe1RtE/fsctpIJZm\nRqqMO190aqEj7jh/0Hq+163VAXc8fbz+Qi9ZDz7axWfV7WGVEMlG/cCDOj08uD5K\n1usg2DVIWjlhp9v0TD29bLYhVSjFel8dkvisV9PMO0oSiMgaRiXgQfh4oFFHgjgi\nmT83rtMwz6Oh581Wok5RaPrSz9EUALhlMagKWpeB7wKBgQD7JHM6tZVQKKi5Dm4G\nITMxoEIgRXtMrqA6ltbvIqMj0R7me/rkkmYxmbiG9ZIzuENBdxP3OcuQMb8u4Ewb\naRnfBWkVsnzspkCVXStLK3FFBmUZltCXR1vXdM0m/0FfQVKHC8E+9QQXpY9j/lxl\nXviE8Dx6R326cZYYjtKZzNRYiwKBgQDgp09qVXcGEhUr05BAxA7/IB3RA5EMSznk\nYTDqVwAg+CBXPWuhcMp7U0Ghy4AaRISie6L/mwJAu9tit+fPmskW4UQO8ah0koc5\ngEDG9egvkeO8VhbSSbOKnBt8ODjuWTyNxt/G7t1Cx7teVjaNNmOLpmUa6YaTjpMm\nLiPHDaLBHwKBgQDlpvI7+Ho+X7/R4XEY59khgzOUwRS8DV911CNdb6YRBZSlHTBT\nPdB3gOMtfnggFVpjDdnTFCWiiOsTmYXe9t/ygccTUTFNUcXzD8ycI3CjtvJSUQgT\nnexM/IDxLS+BRIGL/mLLCFCiswGJZbrl8897+RbEloVRLbccY9YPUC/JewKBgDhQ\nOEwDWG6hIcV2pvZVLx0sw8ydBEL8qGpjIovbeyDIkfLMZqp1R4xadl1EUbxD6KuC\nKn3AMXRgosHAL8+OkVG27gSA9yUhhdwYFiTJh4ZFH70aJ2ZXKzZXo1wkC8FThSZU\n78tCHRycTI99NPd45sMe1sFEQIPEfYZYkodXF7EpAoGABDOY+iD4OgEOAkjC8CNZ\nt4L+MgMRfw9QL17hPKeUnF9jL7idwjtFejZ+gEoa+u1dLjEEbyBPLggxbkhg2vHe\n+Y8N/NwmbJ8DqvyXBrKyye0BuVJ/H1uk77n1NLWmQn0K3DzfZmWUy/nP5+CX4Pqx\nkSSn1vL4ZJdLcgWSmtdqhHg=\n-----END PRIVATE KEY-----\n",
            "client_email": "test-storage@essential-topic-368917.iam.gserviceaccount.com",
            "client_id": "105388027426993183080",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "token_uri": "https://oauth2.googleapis.com/token",
            "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
            "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/test-storage%40essential-topic-368917.iam.gserviceaccount.com"
          }
        "#, Scope::Compute)
            .await
            .expect("build_auth failed");
        assert!(matches!(auth, GcpAuthenticator::Credentials(..)));
    }

    #[tokio::test]
    async fn fails_bad_api_key() {
        let error = build_auth(r#"api_key = "abc%xyz""#)
            .await
            .expect_err("build failed to error");
        assert_downcast_matches!(error, GcpError, GcpError::InvalidApiKey { .. });
    }

    #[tokio::test]
    async fn fails_bad_credentials_json() {
        let error = GcpAuthenticator::from_str(r#"
        {
            "type": "service_account",
            "project_id": "test-project-id"
        }
    "#, Scope::Compute)
            .await
            .expect_err("build failed to error");
        assert_downcast_matches!(error, GcpError, GcpError::InvalidCredentials { .. });
    }

    fn apply_uri(auth: &GcpAuthenticator, uri: &str) -> String {
        let mut uri: Uri = uri.parse().unwrap();
        auth.apply_uri(&mut uri);
        uri.to_string()
    }

    async fn build_auth(toml: &str) -> crate::Result<GcpAuthenticator> {
        let config: GcpAuthConfig = toml::from_str(toml).expect("Invalid TOML");
        config.build(Scope::Compute).await
    }
}
