use vrl::prelude::Function;

pub mod get_pipeline_state_variable;
pub mod mock_user_log;
pub mod set_pipeline_state_variable;
pub mod user_log;

#[cfg(test)]
pub(crate) mod test_util;

/// VRL functions disabled in the Mezmo Vector build. They reach the network
/// (SSRF, VM-673), or expose the worker's environment, host, or local files
/// (LOG-25266).
pub const DISABLED_VRL_FUNCTIONS: &[&str] = &[
    "dns_lookup",
    "encode_proto",
    "get_env_var",
    "get_hostname",
    "http_request",
    "parse_proto",
    "reverse_dns",
    "validate_json_schema",
];

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::disallowed_methods)] // The names come from the vrl stdlib itself.
    fn remove_disabled_strips_every_listed_function() {
        let mut functions = vrl::stdlib::all();
        // A misspelled or renamed entry would silently disable nothing.
        for name in DISABLED_VRL_FUNCTIONS {
            assert!(
                functions.iter().any(|f| f.identifier() == *name),
                "{name} is not a vrl stdlib function"
            );
        }

        remove_disabled(&mut functions);

        for name in DISABLED_VRL_FUNCTIONS {
            assert!(
                !functions.iter().any(|f| f.identifier() == *name),
                "{name} was not removed"
            );
        }
    }
}
