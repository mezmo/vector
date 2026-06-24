use vrl::prelude::Function;

pub mod get_pipeline_state_variable;
pub mod mock_user_log;
pub mod set_pipeline_state_variable;
pub mod user_log;

#[cfg(test)]
pub(crate) mod test_util;

/// VRL functions disabled in the Mezmo Vector build. Some of these can be used
/// in attack vectors for SSRF. See VM-673.
pub const DISABLED_VRL_FUNCTIONS: &[&str] = &["http_request", "dns_lookup"];

/// Removes the Mezmo-disabled functions ([`DISABLED_VRL_FUNCTIONS`]) from a compiler
/// function set. Call after assembling `vrl::stdlib::all()` + extensions at every VRL
/// compile site.
pub fn remove_disabled(functions: &mut Vec<Box<dyn Function>>) {
    functions.retain(|f| !DISABLED_VRL_FUNCTIONS.contains(&f.identifier()));
}

pub fn vrl_functions() -> Vec<Box<dyn Function>> {
    vec![
        Box::new(user_log::UserLog) as _,
        Box::new(get_pipeline_state_variable::GetPipelineStateVariable) as _,
        Box::new(set_pipeline_state_variable::SetPipelineStateVariable) as _,
    ]
}

pub fn cli_vrl_functions() -> Vec<Box<dyn Function>> {
    vec![
        Box::new(mock_user_log::MockUserLog) as _,
        Box::new(get_pipeline_state_variable::GetPipelineStateVariable) as _,
        Box::new(set_pipeline_state_variable::SetPipelineStateVariable) as _,
    ]
}
