use crate::mezmo::MezmoContext;
use vector_lib::enrichment::{
    vrl_util::Error as EnrichmentTableError, Case, Condition, TableRegistry, TableSearch,
};
use vrl::prelude::*;
#[derive(Clone, Copy, Debug)]
pub struct GetPipelineStateVariable;

impl Function for GetPipelineStateVariable {
    fn identifier(&self) -> &'static str {
        "get_pipeline_state_variable"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[Parameter {
            keyword: "name",
            kind: kind::BYTES,
            required: true,
        }]
    }

    fn examples(&self) -> &'static [Example] {
        &[Example {
            title: "get a variable from state",
            source: r#"get_pipeline_state_variable("foo")"#,
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

        let registry = ctx
            .get_external_context_mut::<TableRegistry>()
            .ok_or(Box::new(EnrichmentTableError::TablesNotLoaded) as Box<dyn DiagnosticMessage>)?;
        let enrichment_tables = registry.as_readonly();

        Ok(GetPipelineStateVariableFn {
            name,
            enrichment_tables,
            mezmo_ctx,
        }
        .as_expr())
    }
}

#[derive(Debug, Clone)]
struct GetPipelineStateVariableFn {
    name: Box<dyn Expression>,
    enrichment_tables: TableSearch,
    mezmo_ctx: MezmoContext,
}

impl FunctionExpression for GetPipelineStateVariableFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        let name = KeyString::from(self.name.resolve(ctx)?.to_string_lossy());

        if self.mezmo_ctx.pipeline_id.is_none() {
            // Noop for non-pipeline components
            return Ok(Value::Null);
        }

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

        match self.enrichment_tables.find_table_row(
            "state_variables",
            Case::Sensitive, // unused
            &conditions,
            Some(&[name.to_string()]),
            None, // indexes aren't used
        ) {
            Ok(data) => {
                let data = data.get(&name).unwrap();
                debug!(
                    "get_pipeline_state_variable lookup result: {data:?}  Value: {}",
                    &data
                );
                // The enrichment handles the case where keys aren't found.  If so, it's Value::Null
                Ok(data.clone())
            }
            Err(err) => {
                // This should only happen if the enrichment table is not accessible
                warn!("Returning noop for state_variables '{name}' lookup: {err:?}");
                Ok(Value::Null)
            }
        }
    }

    fn type_def(&self, _state: &TypeState) -> TypeDef {
        TypeDef::null().infallible()
    }
}
