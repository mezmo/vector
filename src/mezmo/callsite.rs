use std::fmt;

/// A callsite
///
/// This is used define a specific site in the code where a logging call is
/// made. This is used to uniquely identify a call.
#[derive(Clone)]
pub struct Callsite(pub &'static str);

impl PartialEq for Callsite {
    fn eq(&self, other: &Callsite) -> bool {
        core::ptr::eq(
            self as *const _ as *const (),
            other as *const _ as *const (),
        )
    }
}

impl Eq for Callsite {}

impl std::hash::Hash for Callsite {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        (self as *const Callsite).hash(state)
    }
}

impl fmt::Debug for Callsite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Callsite({:p})", self.0)
    }
}

/// A callsite identity
///
/// This is used to define a unique identity of a callsite either directly from
/// Rust or in VRL code.
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct CallsiteIdentity {
    pub site: &'static Callsite,
    pub vrl_position: Option<usize>,
}
