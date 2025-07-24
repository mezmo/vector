use crate::{set_pipeline_state_variable, MezmoContext};
use enrichment::{
    vrl_util::Error as EnrichmentTableError, Case, Condition, TableRegistry, TableSearch,
};
use tracing::{debug, warn};
use vrl::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct SetPipelineStateVariable;

impl Function for SetPipelineStateVariable {
    fn identifier(&self) -> &'static str {
        "set_pipeline_state_variable"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[
            Parameter {
                keyword: "name",
                kind: kind::BYTES,
                required: true,
            },
            Parameter {
                keyword: "value",
                kind: kind::ANY,
                required: true,
            },
        ]
    }

    fn examples(&self) -> &'static [Example] {
        &[
            Example {
                title: "set a state variable for the pipeline",
                source: r#"set_pipeline_state_variable("foo", "bar")"#,
                result: Ok("bar"),
            },
            Example {
                title: "set a state variable for the pipeline",
                source: r#"set_pipeline_state_variable("my_num", 123)"#,
                result: Ok("123"),
            },
            Example {
                title: "set a state variable for the pipeline",
                source: r#"set_pipeline_state_variable("my_arr", [1, 2, 3])"#,
                result: Ok("[1, 2, 3]"),
            },
        ]
    }

    fn compile(
        &self,
        _state: &TypeState,
        ctx: &mut FunctionCompileContext,
        arguments: ArgumentList,
    ) -> Compiled {
        let mezmo_ctx = ctx.get_external_context::<MezmoContext>().cloned().unwrap();
        let name = arguments.required("name");
        let value = arguments.required("value");
        let vrl_position = Some(ctx.span().start());

        let registry = ctx
            .get_external_context_mut::<TableRegistry>()
            .ok_or(Box::new(EnrichmentTableError::TablesNotLoaded) as Box<dyn DiagnosticMessage>)?;
        let enrichment_tables = registry.as_readonly();

        Ok(SetPipelineStateVariableFn {
            enrichment_tables,
            mezmo_ctx,
            vrl_position,
            name,
            value,
        }
        .as_expr())
    }
}

pub fn internal_set_pipeline_state_variable(
    mezmo_ctx: &MezmoContext,
    vrl_position: Option<usize>,
    name: String,
    value: Value,
) -> Resolved {
    // Noop for non-pipeline components
    if mezmo_ctx.pipeline_id.is_none() {
        return Ok(Value::Null);
    }

    set_pipeline_state_variable!(Some(mezmo_ctx.clone()), vrl_position, name, value.clone());

    Ok(value)
}

#[derive(Debug, Clone)]
struct SetPipelineStateVariableFn {
    enrichment_tables: TableSearch,
    mezmo_ctx: MezmoContext,
    vrl_position: Option<usize>,
    name: Box<dyn Expression>,
    value: Box<dyn Expression>,
}

impl FunctionExpression for SetPipelineStateVariableFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        // Noop for non-pipeline components
        if self.mezmo_ctx.pipeline_id.is_none() {
            return Ok(Value::Null);
        }

        // determine the current value of the state variable
        // and only bother updating it if its changed
        let conditions = vec![
            Condition::Equals {
                field: "account_id",
                value: Value::from(&self.mezmo_ctx.account_id),
            },
            Condition::Equals {
                field: "pipeline_id",
                value: Value::from(self.mezmo_ctx.pipeline_id.as_ref().unwrap()),
            },
        ];

        let name = self.name.resolve(ctx)?.to_string_lossy().into_owned();
        let value = self.value.resolve(ctx)?;

        match self.enrichment_tables.find_table_row(
            "state_variables",
            Case::Sensitive, // unused
            &conditions,
            Some(&[name.clone()]),
            None, // indexes aren't used
        ) {
            Ok(data) => {
                let name_keystring = KeyString::from(name.clone());
                let data = data.get(&name_keystring).unwrap();
                debug!(
                    "set_pipeline_state_variable lookup result: {data:?}  Value: {}",
                    &data
                );

                // compare the value
                if data.clone() == value.clone() {
                    return Ok(data.clone());
                }
            }
            Err(err) => {
                warn!("set_pipeline_state_variable: Error looking up state_variables '{name}' lookup: {err:?}");
            }
        }

        debug!("Updating state variable '{name}' with value '{value}'");
        internal_set_pipeline_state_variable(&self.mezmo_ctx, self.vrl_position, name, value)
    }

    fn type_def(&self, _state: &TypeState) -> TypeDef {
        TypeDef::null().infallible()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::test_util::create_test_vrl_context;
    use crate::pipeline_state_variable_change_action::PipelineStateVariableChangeActionSubscription;
    use futures_util::StreamExt;
    use serial_test::serial;
    use std::collections::BTreeMap;
    use tokio::{
        select,
        time::{sleep, Duration},
    };
    use uuid::Uuid;
    use vector_core::event::{Event, LogEvent, VrlTarget};
    use vrl::prelude::state::RuntimeState;

    const TEST_VRL_POSITION: Option<usize> = Some(10);

    fn get_mezmo_context() -> MezmoContext {
        let pipeline_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();
        MezmoContext::try_from(format!(
            "v1:js-script:transform:component_id:{}:{}",
            pipeline_id, account_id
        ))
        .unwrap()
    }

    #[test]
    #[serial]
    fn test_internal_no_specific_component() {
        let mezmo_ctx =
            MezmoContext::try_from("v1:remap:transform:shared:system_id".to_string()).unwrap();

        let result = internal_set_pipeline_state_variable(
            &mezmo_ctx,
            TEST_VRL_POSITION,
            "foo".to_string(),
            Value::from("bar".to_string()),
        );
        assert_eq!(result, Ok(Value::Null));
    }

    #[test]
    #[serial]
    fn test_internal_value_set_string() {
        let test_ctx = get_mezmo_context();
        let value = "bar".to_string();

        let result = internal_set_pipeline_state_variable(
            &test_ctx,
            TEST_VRL_POSITION,
            "foo".to_string(),
            Value::from(value.clone()),
        );
        assert_eq!(result, Ok(Value::from(value)));
    }

    #[test]
    #[serial]
    fn test_internal_value_set_number() {
        let test_ctx = get_mezmo_context();
        let value = Value::from(123);

        let result = internal_set_pipeline_state_variable(
            &test_ctx,
            TEST_VRL_POSITION,
            "foo".to_string(),
            value.clone(),
        );
        assert_eq!(result, Ok(value));
    }

    #[test]
    #[serial]
    fn test_internal_value_set_array() {
        let test_ctx = get_mezmo_context();
        let value = vec![1, 2, 3];

        let result = internal_set_pipeline_state_variable(
            &test_ctx,
            TEST_VRL_POSITION,
            "foo".to_string(),
            Value::from(value.clone()),
        );
        assert_eq!(result, Ok(Value::from(value)));
    }

    #[test]
    #[serial]
    fn test_internal_value_update() {
        let test_ctx = get_mezmo_context();

        let value1 = "bar1".to_string();
        let value2 = "bar2".to_string();

        let result = internal_set_pipeline_state_variable(
            &test_ctx,
            TEST_VRL_POSITION,
            "foo".to_string(),
            Value::from(value1.clone()),
        );
        assert_eq!(result, Ok(Value::from(value1)));

        let result = internal_set_pipeline_state_variable(
            &test_ctx,
            TEST_VRL_POSITION,
            "foo".to_string(),
            Value::from(value2.clone()),
        );
        assert_eq!(result, Ok(Value::from(value2)));
    }

    #[tokio::test]
    #[serial]
    async fn test_resolve_no_change_returns_same_value() {
        // catch issued updates to the service
        let mut log_stream =
            PipelineStateVariableChangeActionSubscription::subscribe().into_stream();

        // Setup mock enrichment table with pre-existing value
        let initial_state = BTreeMap::from([("nochange".into(), "abc".into())]);
        let (enrichment_tables, info, tz) = create_test_vrl_context(initial_state);

        // Create the function expression to test
        let function_expression = SetPipelineStateVariableFn {
            enrichment_tables,
            mezmo_ctx: get_mezmo_context(),
            vrl_position: TEST_VRL_POSITION,
            name: Box::new(expression::Literal::from("nochange")),
            value: Box::new(expression::Literal::from("abc")),
        };

        // Setup VRL execution context
        let mut target = VrlTarget::new(Event::from(LogEvent::default()), &info, false);
        let mut runtime_state = RuntimeState::default();
        let mut ctx = Context::new(&mut target, &mut runtime_state, &tz);

        // Call resolve and assert the result
        let result = function_expression.resolve(&mut ctx);

        assert_eq!(result, Ok(Value::from("abc")));

        // no change should have been issued
        let timeout = select! {
            _ = log_stream.next() => false,
            _ = sleep(Duration::from_millis(100)) => true,
        };

        assert!(timeout, "expected a timeout since nothing was sent");
    }

    #[tokio::test]
    #[serial]
    async fn test_resolve_value_change_propegates() {
        // catch issued updates to the service
        let mut log_stream =
            PipelineStateVariableChangeActionSubscription::subscribe().into_stream();

        // Setup mock enrichment table with pre-existing value
        let initial_state = BTreeMap::from([("changevalue".into(), "orig_value".into())]);
        let (enrichment_tables, info, tz) = create_test_vrl_context(initial_state);

        // Create the function expression to test
        let function_expression = SetPipelineStateVariableFn {
            enrichment_tables,
            mezmo_ctx: get_mezmo_context(),
            vrl_position: TEST_VRL_POSITION,
            name: Box::new(expression::Literal::from("changevalue")),
            value: Box::new(expression::Literal::from("new_value")),
        };

        // Setup VRL execution context
        let mut target = VrlTarget::new(Event::from(LogEvent::default()), &info, false);
        let mut runtime_state = RuntimeState::default();
        let mut ctx = Context::new(&mut target, &mut runtime_state, &tz);

        // Call resolve and assert the result
        let result = function_expression.resolve(&mut ctx);

        assert_eq!(result, Ok(Value::from("new_value")));

        // a change should have been issued
        let timeout = select! {
            _ = log_stream.next() => false,
            _ = sleep(Duration::from_millis(100)) => true,
        };

        assert!(!timeout, "expected an update to be propegated");
    }
}
