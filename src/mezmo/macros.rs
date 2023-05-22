/// Constructs a unique callsite
#[macro_export]
macro_rules! callsite {
    ($type:literal) => {{
        static CALLSITE: $crate::mezmo::callsite::Callsite =
            $crate::mezmo::callsite::Callsite(std::concat!($type, " ", file!(), ":", line!()));
        &CALLSITE
    }};
}

/// Creates a user log
///
/// Try not to call this directly, but instead use the level-specific
/// implementations.
#[macro_export]
macro_rules! user_log {
    ("debug", $user_log:expr, $message:expr, $rate_limit_secs:expr, $vrl_position:expr) => {{
        use $crate::mezmo::callsite::{Callsite, CallsiteIdentity};
        static CALLSITE: &'static Callsite = $crate::callsite!("user_log");
        $user_log.debug(
            $message,
            $rate_limit_secs,
            CallsiteIdentity {
                site: CALLSITE,
                vrl_position: $vrl_position,
            },
        );
    }};
    ("info", $user_log:expr, $message:expr, $rate_limit_secs:expr, $vrl_position:expr) => {{
        use $crate::mezmo::callsite::{Callsite, CallsiteIdentity};
        static CALLSITE: &'static Callsite = $crate::callsite!("user_log");
        $user_log.info(
            $message,
            $rate_limit_secs,
            CallsiteIdentity {
                site: CALLSITE,
                vrl_position: $vrl_position,
            },
        );
    }};
    ("warn", $user_log:expr, $message:expr, $rate_limit_secs:expr, $vrl_position:expr) => {{
        use $crate::mezmo::callsite::{Callsite, CallsiteIdentity};
        static CALLSITE: &'static Callsite = $crate::callsite!("user_log");
        $user_log.warn(
            $message,
            $rate_limit_secs,
            CallsiteIdentity {
                site: CALLSITE,
                vrl_position: $vrl_position,
            },
        );
    }};
    ("error", $user_log:expr, $message:expr, $rate_limit_secs:expr, $vrl_position:expr) => {{
        use $crate::mezmo::callsite::{Callsite, CallsiteIdentity};
        static CALLSITE: &'static Callsite = $crate::callsite!("user_log");
        $user_log.error(
            $message,
            $rate_limit_secs,
            CallsiteIdentity {
                site: CALLSITE,
                vrl_position: $vrl_position,
            },
        );
    }};
}

#[macro_export]
macro_rules! user_log_debug {
    ($user_log:expr, $message:expr, rate_limit_secs: $rate_limit_secs:expr) => {{
        $crate::user_log!("debug", $user_log, $message, Some($rate_limit_secs), None);
    }};
    ($user_log:expr, $message:expr) => {{
        $crate::user_log!("debug", $user_log, $message, None, None);
    }};
}

#[macro_export]
macro_rules! user_log_info {
    ($user_log:expr, $message:expr, rate_limit_secs: $rate_limit_secs:expr) => {{
        $crate::user_log!("info", $user_log, $message, Some($rate_limit_secs), None);
    }};
    ($user_log:expr, $message:expr) => {{
        $crate::user_log!("info", $user_log, $message, None, None);
    }};
}

#[macro_export]
macro_rules! user_log_warn {
    ($user_log:expr, $message:expr, rate_limit_secs: $rate_limit_secs:expr) => {{
        $crate::user_log!("warn", $user_log, $message, Some($rate_limit_secs), None);
    }};
    ($user_log:expr, $message:expr) => {{
        $crate::user_log!("warn", $user_log, $message, None, None);
    }};
}

#[macro_export]
macro_rules! user_log_error {
    ($user_log:expr, $message:expr, rate_limit_secs: $rate_limit_secs:expr) => {{
        $crate::user_log!("error", $user_log, $message, Some($rate_limit_secs), None);
    }};
    ($user_log:expr, $message:expr) => {{
        $crate::user_log!("error", $user_log, $message, None, None);
    }};
}
