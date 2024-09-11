use vrl::prelude::*;
use vrl::stdlib::Log;

#[derive(Clone, Copy, Debug)]
pub struct MockUserLog;

impl Function for MockUserLog {
    fn identifier(&self) -> &'static str {
        "user_log"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[
            Parameter {
                keyword: "value",
                kind: kind::ANY,
                required: true,
            },
            Parameter {
                keyword: "level",
                kind: kind::BYTES,
                required: false,
            },
            Parameter {
                keyword: "rate_limit_secs",
                kind: kind::INTEGER,
                required: false,
            },
        ]
    }

    fn examples(&self) -> &'static [Example] {
        &[
            Example {
                title: "default log level (info)",
                source: r#"user_log("foo")"#,
                result: Ok("null"),
            },
            Example {
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
