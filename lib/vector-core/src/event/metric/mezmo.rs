#[derive(Debug)]
pub enum TransformError {
    FieldNotFound { field: String },
    FieldInvalidType { field: String },
    InvalidMetricType { type_name: String },
    InvalidKind { kind: String },
    FieldNull { field: String },
    ParseIntOverflow { field: String },
}
