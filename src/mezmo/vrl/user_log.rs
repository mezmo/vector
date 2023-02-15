use crate::mezmo::{user_trace::MezmoUserLog, MezmoContext};
use ::value::Value;
use bytes::Bytes;
use vrl::{
    function::{ArgumentList, Compiled, Example, FunctionCompileContext, Parameter},
    prelude::*,
    state::TypeState,
    value::{kind, VrlValueConvert},
    Context, Expression, Function,
};

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
        _state: &TypeState,
        ctx: &mut FunctionCompileContext,
        arguments: ArgumentList,
    ) -> Compiled {
        let mezmo_ctx = ctx.get_external_context::<MezmoContext>().cloned();
        let value = arguments.required("value");
        let valid_levels = vec!["debug".into(), "info".into(), "warn".into(), "error".into()];
        let level = arguments
            .optional_enum("level", &valid_levels)?
            .unwrap_or_else(|| "info".into())
            .try_bytes()
            .expect("log level not bytes");

        Ok(UserLogFn {
            mezmo_ctx,
            value,
            level,
        }
        .as_expr())
    }
}

#[derive(Debug, Clone)]
struct UserLogFn {
    mezmo_ctx: Option<MezmoContext>,
    value: Box<dyn Expression>,
    level: Bytes,
}

impl FunctionExpression for UserLogFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        let value = self.value.resolve(ctx)?;
        match self.level.as_ref() {
            b"debug" => self.mezmo_ctx.debug(value),
            b"warn" => self.mezmo_ctx.warn(value),
            b"error" => self.mezmo_ctx.error(value),
            _ => self.mezmo_ctx.info(value),
        }
        Ok(Value::Null)
    }

    fn type_def(&self, _state: &TypeState) -> TypeDef {
        TypeDef::null().infallible()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use vector_common::TimeZone;
    use vector_core::event::{LogEvent, VrlTarget};
    use vrl::{CompileConfig, ProgramInfo};

    #[test]
    fn user_log_doesnotcrash() {
        let state = TypeState::default();
        let mut compile_ctx =
            FunctionCompileContext::new(vrl::diagnostic::Span::new(0, 0), CompileConfig::default());

        let user_log = UserLog {};
        let args: HashMap<&'static str, ::value::Value> =
            vec![("value", 42.into()), ("level", "warn".into())]
                .into_iter()
                .collect();
        let expression = user_log
            .compile(&state, &mut compile_ctx, args.into())
            .expect("expression should compile");

        let program_info = ProgramInfo {
            fallible: false,
            abortable: false,
            target_queries: vec![],
            target_assignments: vec![],
        };
        let event = LogEvent::default();
        let mut target = VrlTarget::new(event.into(), &program_info, false);
        let mut runtime_state = state::Runtime::default();
        let mut ctx = Context::new(&mut target, &mut runtime_state, &TimeZone::Local);

        let res = expression.resolve(&mut ctx);
        assert_eq!(res, Ok(Value::Null));

        let res_tdef = expression.type_def(&state);
        assert_eq!(res_tdef, TypeDef::null().infallible());
    }
}
