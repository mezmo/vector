use crate::{MezmoContext, set_pipeline_state_variable};
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
                kind: kind::BYTES,
                required: true,
            },
        ]
    }

    fn examples(&self) -> &'static [Example] {
        &[Example {
            title: "set a state variable for the pipeline",
            source: r#"set_pipeline_state_variable("foo", "bar")"#,
            result: Ok("bar"),
        }]
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
    value: String
) -> Resolved {

    // Noop for non-pipeline components
    if mezmo_ctx.pipeline_id.is_none() {
        return Ok(Value::Null);
    }

    set_pipeline_state_variable!(
        mezmo_ctx,
        vrl_position,
        name,
        value
    );
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
        let value = self.value.resolve(ctx)?.to_string_lossy().into_owned();

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
                if data.clone() != Value::from(value.clone()) {
                    return Ok(Value::Null)
                }
            }
            Err(err) => {
                warn!("set_pipeline_state_variable: Error looking up state_variables '{name}' lookup: {err:?}");
            }
        }

        debug!("Updating state variable '{name}' with value '{value}'");
        return internal_set_pipeline_state_variable(
            &self.mezmo_ctx,
            self.vrl_position,
            name,
            value,
        );
    }

    fn type_def(&self, _state: &TypeState) -> TypeDef {
        TypeDef::null().infallible()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_VRL_POSITION: Option<usize> = Some(10);

    fn get_mezmo_context() -> MezmoContext {
        MezmoContext::try_from(
            "v1:js-script:transform:component_id:pipeline_id:cea71e55-a1ec-4e5f-a5c0-c0e10b1a571c"
                .to_string(),
        )
        .unwrap()
    }

    #[test]
    fn test_no_context() {
        let mezmo_ctx = MezmoContext::try_from("v1:remap:transform:shared:pipeline_id".to_string())
            .unwrap();

        let result = internal_set_pipeline_state_variable(
            &mezmo_ctx,
            TEST_VRL_POSITION,
            "foo".to_string(),
            "bar".to_string(),
        );
        assert_eq!(result, Ok(Value::Null));
    }

    #[test]
    fn test_value_initial_set() {
        let test_ctx = get_mezmo_context();
        let value = "bar".to_string();

        let result = internal_set_pipeline_state_variable(
            &test_ctx,
            TEST_VRL_POSITION,
            "foo".to_string(),
            value.clone(),
        );
        assert_eq!(result, Ok(Value::from(value)));
    }

    #[test]
    fn test_value_update() {
        let test_ctx = get_mezmo_context();

        let value1 = "bar1".to_string();
        let value2 = "bar2".to_string();

        let result = internal_set_pipeline_state_variable(
            &test_ctx,
            TEST_VRL_POSITION,
            "foo".to_string(),
            value1.clone(),
        );
        assert_eq!(result, Ok(Value::from(value1)));

        let result = internal_set_pipeline_state_variable(
            &test_ctx,
            TEST_VRL_POSITION,
            "foo".to_string(),
            value2.clone(),
        );
        assert_eq!(result, Ok(Value::from(value2)));
    }
}
