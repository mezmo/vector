/// Constructs a unique callsite
#[macro_export]
macro_rules! callsite {
    ($type:literal) => {{
        static CALLSITE: $crate::callsite::Callsite =
            $crate::callsite::Callsite(std::concat!($type, " ", file!(), ":", line!()));
        &CALLSITE
    }};
}

/// Creates a user log
///
/// Try not to call this directly, but instead use the level-specific
/// implementations if calling `user_log` from within Vector. If calling via the
/// VRL implementation, it will automatically use this main `user_log` macro, and NOT
/// the overloaded functions defined below.
#[macro_export]
macro_rules! user_log {
    ("debug", $user_log:expr, $message:expr, $rate_limit_secs:expr, $captured_data:expr, $vrl_position:expr) => {{
        use $crate::callsite::{Callsite, CallsiteIdentity};
        static CALLSITE: &'static Callsite = $crate::callsite!("user_log");
        $user_log.debug(
            $message,
            $rate_limit_secs,
            $captured_data,
            CallsiteIdentity {
                site: CALLSITE,
                vrl_position: $vrl_position,
            },
        );
    }};
    ("info", $user_log:expr, $message:expr, $rate_limit_secs:expr, $captured_data:expr, $vrl_position:expr) => {{
        use $crate::callsite::{Callsite, CallsiteIdentity};
        static CALLSITE: &'static Callsite = $crate::callsite!("user_log");
        $user_log.info(
            $message,
            $rate_limit_secs,
            $captured_data,
            CallsiteIdentity {
                site: CALLSITE,
                vrl_position: $vrl_position,
            },
        );
    }};
    ("warn", $user_log:expr, $message:expr, $rate_limit_secs:expr, $captured_data:expr, $vrl_position:expr) => {{
        use $crate::callsite::{Callsite, CallsiteIdentity};
        static CALLSITE: &'static Callsite = $crate::callsite!("user_log");
        $user_log.warn(
            $message,
            $rate_limit_secs,
            $captured_data,
            CallsiteIdentity {
                site: CALLSITE,
                vrl_position: $vrl_position,
            },
        );
    }};
    ("error", $user_log:expr, $message:expr, $rate_limit_secs:expr, $captured_data:expr, $vrl_position:expr) => {{
        use $crate::callsite::{Callsite, CallsiteIdentity};
        static CALLSITE: &'static Callsite = $crate::callsite!("user_log");
        $user_log.error(
            $message,
            $rate_limit_secs,
            $captured_data,
            CallsiteIdentity {
                site: CALLSITE,
                vrl_position: $vrl_position,
            },
        );
    }};
}

#[macro_export]
macro_rules! user_log_debug {
    // Specifying the optional argument: `rate_limit_secs`
    ($user_log:expr, $message:expr, rate_limit_secs: $rate_limit_secs:expr) => {{
        $crate::user_log!(
            "debug",
            $user_log,
            $message,
            Some($rate_limit_secs),
            None,
            None
        );
    }};
    // only the required arguments; No optional args.
    ($user_log:expr, $message:expr) => {{
        $crate::user_log!("debug", $user_log, $message, None, None, None);
    }};
}

#[macro_export]
macro_rules! user_log_info {
    ($user_log:expr, $message:expr, rate_limit_secs: $rate_limit_secs:expr) => {{
        $crate::user_log!(
            "info",
            $user_log,
            $message,
            Some($rate_limit_secs),
            None,
            None
        );
    }};
    ($user_log:expr, $message:expr) => {{
        $crate::user_log!("info", $user_log, $message, None, None, None);
    }};
}

#[macro_export]
macro_rules! user_log_warn {
    ($user_log:expr, $message:expr, rate_limit_secs: $rate_limit_secs:expr) => {{
        $crate::user_log!(
            "warn",
            $user_log,
            $message,
            Some($rate_limit_secs),
            None,
            None
        );
    }};
    // Overload for the common case of only specifying `captured_data`
    ($user_log:expr, $message:expr, captured_data: $captured_data:expr) => {{
        $crate::user_log!(
            "warn",
            $user_log,
            $message,
            None,
            Some($captured_data),
            None
        );
    }};
    ($user_log:expr, $message:expr) => {{
        $crate::user_log!("warn", $user_log, $message, None, None, None);
    }};
}

#[macro_export]
macro_rules! user_log_error {
    ($user_log:expr, $message:expr, rate_limit_secs: $rate_limit_secs:expr) => {{
        $crate::user_log!(
            "error",
            $user_log,
            $message,
            Some($rate_limit_secs),
            None,
            None
        );
    }};
    // Overload for the common case of only specifying `captured_data`
    ($user_log:expr, $message:expr, captured_data: $captured_data:expr) => {{
        $crate::user_log!(
            "error",
            $user_log,
            $message,
            None,
            Some($captured_data),
            None
        );
    }};
    ($user_log:expr, $message:expr) => {{
        $crate::user_log!("error", $user_log, $message, None, None, None);
    }};
}

#[macro_export]
macro_rules! set_pipeline_state_variable {
    ($mezmo_ctx:expr, $vrl_position:expr, $name:expr, $value:expr) => {
        {
            use $crate::callsite::{Callsite, CallsiteIdentity};
            use $crate::pipeline_state_variable_change_action::PipelineStateVariableChangeActionLog;

            static CALLSITE: &'static Callsite = $crate::callsite!("set_pipeline_state_variable");

            $mezmo_ctx.clone().set_pipeline_state_variable(
                $name,
                $value,
                CallsiteIdentity {
                    site: CALLSITE,
                    vrl_position: $vrl_position,
                },
            );
        }
    };
}
