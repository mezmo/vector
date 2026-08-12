use vrl::prelude::*;
use vrl::stdlib::Log;

#[derive(Clone, Copy, Debug)]
pub struct MockUserLog;

impl Function for MockUserLog {
    fn identifier(&self) -> &'static str {
        "user_log"
    }

    fn usage(&self) -> &'static str {
        "Mock of `user_log` used by the VRL CLI; emits a standard log line instead of a Mezmo user log."
    }

    fn category(&self) -> &'static str {
        Category::System.as_ref()
    }

    fn return_kind(&self) -> u16 {
        kind::NULL
    }

    fn parameters(&self) -> &'static [Parameter] {
        const PARAMETERS: &[Parameter] = &[
            Parameter::required("value", kind::ANY, "The message value to log."),
            Parameter::optional(
                "level",
                kind::BYTES,
                "The log level: debug, info, warn, or error. Defaults to info.",
            ),
            Parameter::optional(
                "rate_limit_secs",
                kind::INTEGER,
                "Rate limit for the log message, in seconds.",
            ),
        ];
        PARAMETERS
    }

    fn examples(&self) -> &'static [Example] {
        &[
            example! {
                title: "default log level (info)",
                source: r#"user_log("foo")"#,
                result: Ok("null"),
            },
            example! {
                title: "custom level",
                source: r#"user_log("foo", "error")"#,
                result: Ok("null"),
            },
        ]
    }

    fn compile(
        &self,
        state: &TypeState,
        ctx: &mut FunctionCompileContext,
        arguments: ArgumentList,
    ) -> Compiled {
        let normal_log = Log;
        normal_log.compile(state, ctx, arguments)
    }
}
