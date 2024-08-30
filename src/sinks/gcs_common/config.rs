use futures::FutureExt;
use http::{StatusCode, Uri};
use hyper::Body;
use snafu::Snafu;
use vector_lib::configurable::configurable_component;
use vrl::value::Value;

use crate::{
    gcp::{GcpAuthenticator, GcpError},
    http::HttpClient,
    sinks::{
        gcs_common::service::GcsResponse,
        util::retries::{RetryAction, RetryLogic},
        Healthcheck, HealthcheckError,
    },
};
use mezmo::{user_log_error, user_trace::MezmoUserLog, MezmoContext};

pub const BASE_URL: &str = "https://storage.googleapis.com/";

/// GCS Predefined ACLs.
///
/// For more information, see [Predefined ACLs][predefined_acls].
///
/// [predefined_acls]: https://cloud.google.com/storage/docs/access-control/lists#predefined-acl
#[configurable_component]
#[derive(Clone, Copy, Debug, Derivative)]
#[derivative(Default)]
#[serde(rename_all = "kebab-case")]
pub enum GcsPredefinedAcl {
    /// Bucket/object can be read by authenticated users.
    ///
    /// The bucket/object owner is granted the `OWNER` permission, and anyone authenticated Google
    /// account holder is granted the `READER` permission.
    AuthenticatedRead,

    /// Object is semi-private.
    ///
    /// Both the object owner and bucket owner are granted the `OWNER` permission.
    ///
    /// Only relevant when specified for an object: this predefined ACL is otherwise ignored when
    /// specified for a bucket.
    BucketOwnerFullControl,

    /// Object is private, except to the bucket owner.
    ///
    /// The object owner is granted the `OWNER` permission, and the bucket owner is granted the
    /// `READER` permission.
    ///
    /// Only relevant when specified for an object: this predefined ACL is otherwise ignored when
    /// specified for a bucket.
    BucketOwnerRead,

    /// Bucket/object are private.
    ///
    /// The bucket/object owner is granted the `OWNER` permission, and no one else has
    /// access.
    Private,

    /// Bucket/object are private within the project.
    ///
    /// Project owners and project editors are granted the `OWNER` permission, and anyone who is
    /// part of the project team is granted the `READER` permission.
    ///
    /// This is the default.
    #[derivative(Default)]
    ProjectPrivate,

    /// Bucket/object can be read publically.
    ///
    /// The bucket/object owner is granted the `OWNER` permission, and all other users, whether
    /// authenticated or anonymous, are granted the `READER` permission.
    PublicRead,
}

/// GCS storage classes.
///
/// For more information, see [Storage classes][storage_classes].
///
/// [storage_classes]: https://cloud.google.com/storage/docs/storage-classes
#[configurable_component]
#[derive(Clone, Copy, Debug, Derivative, PartialEq, Eq)]
#[derivative(Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GcsStorageClass {
    /// Standard storage.
    ///
    /// This is the default.
    #[derivative(Default)]
    Standard,

    /// Nearline storage.
    Nearline,

    /// Coldline storage.
    Coldline,

    /// Archive storage.
    Archive,
}

#[derive(Debug, Snafu)]
pub enum GcsError {
    #[snafu(display("Bucket {:?} not found", bucket))]
    BucketNotFound { bucket: String },

    #[snafu(display("The authentication provided is invalid"))]
    InvalidAuth,
}

pub fn build_healthcheck(
    bucket: String,
    client: HttpClient,
    base_url: String,
    auth: Option<GcpAuthenticator>,
    mezmo_ctx: Option<MezmoContext>,
) -> crate::Result<Healthcheck> {
    let healthcheck = async move {
        let uri = base_url.parse::<Uri>()?;
        let mut request = http::Request::head(uri).body(Body::empty())?;

        match auth {
            Some(auth) => {
                auth.apply(&mut request);

                let not_found_error = GcsError::BucketNotFound { bucket }.into();

                let response = client.send(request).await.map_err(|error| {
                    user_log_error!(mezmo_ctx, Value::from(format!("{error}")));
                    error
                })?;
                healthcheck_response(response, not_found_error)
            }
            None => {
                let err = GcsError::InvalidAuth.into();
                user_log_error!(mezmo_ctx, Value::from(format!("{err}")));
                Err(err)
            }
        }
    };

    Ok(healthcheck.boxed())
}

pub fn healthcheck_response(
    response: http::Response<hyper::Body>,
    not_found_error: crate::Error,
) -> crate::Result<()> {
    match response.status() {
        StatusCode::OK => Ok(()),
        StatusCode::FORBIDDEN => Err(GcpError::HealthcheckForbidden.into()),
        StatusCode::NOT_FOUND => Err(not_found_error),
        status => Err(HealthcheckError::UnexpectedStatus { status }.into()),
    }
}

#[derive(Clone)]
pub struct GcsRetryLogic;

// This is a clone of HttpRetryLogic for the Body type, should get merged
impl RetryLogic for GcsRetryLogic {
    type Error = hyper::Error;
    type Response = GcsResponse;

    fn is_retriable_error(&self, _error: &Self::Error) -> bool {
        true
    }

    fn should_retry_response(&self, response: &Self::Response) -> RetryAction {
        if let Some(inner) = &response.inner {
            let status = inner.status();

            return match status {
                StatusCode::UNAUTHORIZED => RetryAction::Retry("unauthorized".into()),
                StatusCode::TOO_MANY_REQUESTS => RetryAction::Retry("too many requests".into()),
                StatusCode::NOT_IMPLEMENTED => {
                    RetryAction::DontRetry("endpoint not implemented".into())
                }
                _ if status.is_server_error() => RetryAction::Retry(status.to_string().into()),
                _ if status.is_success() => RetryAction::Successful,
                _ => RetryAction::DontRetry(format!("response status: {}", status).into()),
            };
        }

        RetryAction::DontRetry("response unavilable".to_string().into())
    }
}
