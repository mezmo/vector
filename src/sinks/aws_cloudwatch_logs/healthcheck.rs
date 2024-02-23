use aws_sdk_cloudwatchlogs::error::DescribeLogGroupsError;
use aws_sdk_cloudwatchlogs::types::SdkError;
use aws_sdk_cloudwatchlogs::Client as CloudwatchLogsClient;
use snafu::Snafu;

use crate::{
    config::SinkContext, mezmo::user_trace::MezmoUserLog,
    sinks::aws_cloudwatch_logs::config::CloudwatchLogsSinkConfig, user_log_error,
};
use vrl::value::Value;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Snafu)]
enum HealthcheckError {
    #[snafu(display("DescribeLogGroups failed: {}", source))]
    DescribeLogGroupsFailed {
        source: SdkError<DescribeLogGroupsError>,
    },
    #[snafu(display("No log group found"))]
    NoLogGroup,
    #[snafu(display("Unable to extract group name"))]
    GroupNameError,
    #[snafu(display("Group name mismatch: expected {}, found {}", expected, name))]
    GroupNameMismatch { expected: String, name: String },
}

pub async fn healthcheck(
    config: CloudwatchLogsSinkConfig,
    client: CloudwatchLogsClient,
    cx: SinkContext,
) -> crate::Result<()> {
    let group_name = config.group_name.get_ref().to_owned();
    let expected_group_name = group_name.clone();

    // This will attempt to find the group name passed in and verify that
    // it matches the one that AWS sends back.
    let result = client
        .describe_log_groups()
        .limit(1)
        .log_group_name_prefix(group_name)
        .send()
        .await;

    match result {
        Ok(resp) => match resp.log_groups.and_then(|g| g.into_iter().next()) {
            Some(group) => {
                if let Some(name) = group.log_group_name {
                    if name == expected_group_name {
                        Ok(())
                    } else {
                        let err = HealthcheckError::GroupNameMismatch {
                            expected: expected_group_name,
                            name,
                        };
                        user_log_error!(cx.mezmo_ctx, Value::from(format!("{}", err)));
                        Err(err.into())
                    }
                } else {
                    user_log_error!(
                        cx.mezmo_ctx,
                        Value::from(format!("{}", HealthcheckError::GroupNameError))
                    );
                    Err(HealthcheckError::GroupNameError.into())
                }
            }
            None => {
                if config.group_name.is_dynamic() {
                    info!("Skipping healthcheck log group check: `group_name` is dynamic.");
                    Ok(())
                } else if config.create_missing_group {
                    info!("Skipping healthcheck log group check: `group_name` will be created if missing.");
                    Ok(())
                } else {
                    user_log_error!(
                        cx.mezmo_ctx,
                        Value::from(format!("{}", HealthcheckError::NoLogGroup))
                    );
                    Err(HealthcheckError::NoLogGroup.into())
                }
            }
        },
        Err(source) => {
            let err = HealthcheckError::DescribeLogGroupsFailed { source };
            user_log_error!(cx.mezmo_ctx, Value::from(format!("{}", err)));
            Err(err.into())
        }
    }
}
