pub mod user_log;

pub fn vrl_functions() -> Vec<Box<dyn vrl::Function>> {
    vec![Box::new(user_log::UserLog) as _]
}
