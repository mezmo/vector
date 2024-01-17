use crate::{
    mezmo::{user_trace::MezmoUserLog, MezmoContext},
    user_log,
};
use bytes::Bytes;
use vrl::{prelude::*, value::Value};

#[derive(Clone, Copy, Debug)]
pub struct UserLog;

impl Function for UserLog {
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
        let mezmo_ctx = ctx.get_external_context::<MezmoContext>().cloned();
        let value = arguments.required("value");
        let valid_levels = vec!["debug".into(), "info".into(), "warn".into(), "error".into()];
        let level = arguments
            .optional_enum("level", &valid_levels, state)?
            .unwrap_or_else(|| "info".into())
            .try_bytes()
            .expect("log level not bytes");
        let rate_limit_secs = arguments.optional("rate_limit_secs");
        let vrl_position = ctx.span().start();

        Ok(UserLogFn {
            mezmo_ctx,
            value,
            level,
            rate_limit_secs,
            vrl_position,
        }
        .as_expr())
    }
}

#[derive(Debug, Clone)]
struct UserLogFn {
    mezmo_ctx: Option<MezmoContext>,
    value: Box<dyn Expression>,
    level: Bytes,
    rate_limit_secs: Option<Box<dyn Expression>>,
    vrl_position: usize,
}

impl FunctionExpression for UserLogFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        let mezmo_ctx = &self.mezmo_ctx;
        let value = self.value.resolve(ctx)?;
        let rate_limit_secs = match &self.rate_limit_secs {
            Some(expr) => u64::try_from(expr.resolve(ctx)?.try_integer()?).ok(),
            None => None,
        };
        let vrl_position = Some(self.vrl_position);
        match self.level.as_ref() {
            b"debug" => {
                user_log!("debug", mezmo_ctx, value, rate_limit_secs, vrl_position);
            }
            b"warn" => {
                user_log!("warn", mezmo_ctx, value, rate_limit_secs, vrl_position);
            }
            b"error" => {
                user_log!("error", mezmo_ctx, value, rate_limit_secs, vrl_position);
            }
            _ => {
                user_log!("info", mezmo_ctx, value, rate_limit_secs, vrl_position);
            }
        }
        Ok(Value::Null)
    }

    fn type_def(&self, _state: &TypeState) -> TypeDef {
        TypeDef::null().infallible()
    }
}
// Disable until we address LOG-16814
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::collections::HashMap;
//     use vector_common::TimeZone;
//     use vector_lib::event::{LogEvent, VrlTarget};
//     use vrl::{CompileConfig, ProgramInfo};

//     #[test]
//     fn user_log_doesnotcrash() {
//         let state = TypeState::default();
//         let mut compile_ctx =
//             FunctionCompileContext::new(vrl::diagnostic::Span::new(0, 0), CompileConfig::default());

//         let user_log = UserLog {};
//         let args: HashMap<&'static str, ::value::Value> =
//             vec![("value", 42.into()), ("level", "warn".into())]
//                 .into_iter()
//                 .collect();
//         let expression = user_log
//             .compile(
//                 &state,
//                 &mut compile_ctx,
//                 ArgumentList {
//                     arguments: args
//                         .into_iter()
//                         .map(|(k, v)| (k, v.into()))
//                         .collect::<HashMap<_, _>>(),
//                     closure: None,
//                 },
//             )
//             .expect("expression should compile");

//         let program_info = ProgramInfo {
//             fallible: false,
//             abortable: false,
//             target_queries: vec![],
//             target_assignments: vec![],
//         };
//         let event = LogEvent::default();
//         let mut target = VrlTarget::new(event.into(), &program_info, false);
//         let mut runtime_state = state::Runtime::default();
//         let mut ctx = Context::new(&mut target, &mut runtime_state, &TimeZone::Local);

//         let res = expression.resolve(&mut ctx);
//         assert_eq!(res, Ok(Value::Null));

//         let res_tdef = expression.type_def(&state);
//         assert_eq!(res_tdef, TypeDef::null().infallible());
//     }
// }
