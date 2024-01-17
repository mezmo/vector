use vrl::prelude::Function;

pub mod get_pipeline_state_variable;
pub mod mock_user_log;
pub mod user_log;

pub fn vrl_functions() -> Vec<Box<dyn Function>> {
    vec![
        Box::new(user_log::UserLog) as _,
        Box::new(get_pipeline_state_variable::GetPipelineStateVariable) as _,
    ]
}

pub fn cli_vrl_functions() -> Vec<Box<dyn Function>> {
    vec![
        Box::new(mock_user_log::MockUserLog) as _,
        Box::new(get_pipeline_state_variable::GetPipelineStateVariable) as _,
    ]
}
