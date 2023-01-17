use snafu::Snafu;
use std::convert::Infallible;
use std::str::FromStr;
use value::Value;

#[allow(dead_code)]
pub mod user_trace;

#[derive(Debug, Snafu)]
pub enum ContextParseError {
    #[snafu(display("{id} is not in user-facing component identifier format."))]
    NotUserIdentifier { id: String },
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum ContextIdentifier {
    Value { id: String },
    Shared,
}

impl FromStr for ContextIdentifier {
    type Err = Infallible;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        if id == "shared" {
            Ok(Self::Shared)
        } else {
            Ok(Self::Value { id: id.to_owned() })
        }
    }
}

impl From<&ContextIdentifier> for Value {
    fn from(ctx: &ContextIdentifier) -> Self {
        match ctx {
            ContextIdentifier::Value { id } => Value::from(id.as_str()),
            ContextIdentifier::Shared => Value::from("shared"),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub enum ComponentKind {
    Source,
    Transform,
    Sink,
}

impl From<&ComponentKind> for Value {
    fn from(kind: &ComponentKind) -> Self {
        match kind {
            ComponentKind::Source => Value::from("source"),
            ComponentKind::Sink => Value::from("sink"),
            ComponentKind::Transform => Value::from("transform"),
        }
    }
}

/// Container for Mezmo-specific information that may be injected into a source, transform
/// or sink context as long as the identifier for that component follows our naming pattern
/// used when we generate pipeline components.
#[derive(Debug, Clone)]
pub struct MezmoContext {
    account_id: ContextIdentifier,
    pipeline_id: ContextIdentifier,
    component_id: String,
    component_kind: ComponentKind,
    internal: bool,
}

impl TryFrom<String> for MezmoContext {
    type Error = ContextParseError;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        let mut parts: Vec<&str> = id.split(":").collect();
        if parts.len() == 4 && parts[3] == "shared" {
            // Format: {version}:{kind}:{component_id}:shared
            // Example: 'v1:internal_source:metrics:shared'
            //
            // To normalize the internal component id scheme into a user-defined id scheme,
            // we need to insert the type field and then insert the account_id, which should be
            // the same "shared" string as the pipeline_id.
            parts.insert(1, "");
            parts.push("shared");
        }

        if parts.len() != 6 || parts[0] != "v1" {
            return Err(Self::Error::NotUserIdentifier { id });
        }

        // Format: 'v1:{type}:{kind}:{component_id}:{pipeline_id}:{account_id}'
        // Example: v1:mezmo:sink:ef757476-43a5-4e0d-b998-3db35dbde001:1515707f-f668-4ca1-8493-969e5b13e781:800e5a08-3e67-431c-bbf0-14aa94beafcc
        let account_id = ContextIdentifier::from_str(parts[5]).unwrap();
        let pipeline_id = ContextIdentifier::from_str(parts[4]).unwrap();
        let component_id = parts[3].to_owned();
        let internal = parts[2].starts_with("internal_");
        let component_kind = match parts[2] {
            "source" | "internal_source" => ComponentKind::Source,
            "sink" | "internal_sink" => ComponentKind::Sink,
            "transform" | "internal_transform" => ComponentKind::Transform,
            _ => return Err(Self::Error::NotUserIdentifier { id }),
        };

        Ok(Self {
            account_id,
            pipeline_id,
            component_id,
            component_kind,
            internal,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mezmo_context_try_from_shared_component() {
        let res = MezmoContext::try_from("v1:internal_source:http:shared".to_owned()).unwrap();
        assert_eq!(res.account_id, ContextIdentifier::Shared);
        assert_eq!(res.pipeline_id, ContextIdentifier::Shared);
        assert_eq!(res.component_kind, ComponentKind::Source);
        assert_eq!(res.component_id, "http".to_owned());
        assert!(res.internal);
    }

    #[test]
    fn test_mezmo_context_try_from_internal_component() {
        let res = MezmoContext::try_from(
            "v1:kafka:internal_source:component_id:pipeline_id:account_id".to_owned(),
        )
        .unwrap();
        assert!(matches!(res.account_id, ContextIdentifier::Value { id } if id == "account_id"));
        assert!(matches!(res.pipeline_id, ContextIdentifier::Value { id } if id == "pipeline_id"));
        assert_eq!(res.component_kind, ComponentKind::Source);
        assert_eq!(res.component_id, "component_id".to_owned());
        assert!(res.internal);
    }

    #[test]
    fn test_mezmo_context_try_from_user_component() {
        let res =
            MezmoContext::try_from("v1:mezmo:sink:component_id:pipeline_id:account_id".to_owned())
                .unwrap();
        assert!(matches!(res.account_id, ContextIdentifier::Value { id } if id == "account_id"));
        assert!(matches!(res.pipeline_id, ContextIdentifier::Value { id } if id == "pipeline_id"));
        assert_eq!(res.component_kind, ComponentKind::Sink);
        assert_eq!(res.component_id, "component_id".to_owned());
        assert!(!res.internal);
    }

    #[test]
    fn test_mezmo_context_try_from_invalid_case_1() {
        let invalid_formats = vec![
            "not even close to a valid format",
            "v1: this just happens to start with a valid prefix",
            "url:mezmo:sink:component_id:pipeline_id:account_id",
            "v2:mezmo:sink:component_id:pipeline_id:account_id",
            "",
        ];

        for case in invalid_formats {
            let res = MezmoContext::try_from(case.to_owned());
            assert!(
                matches!(res, Err(ContextParseError::NotUserIdentifier { id: _ })),
                "{} was not rejected as NotUserIdentifier",
                case
            );
        }
    }
}
