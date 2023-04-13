use std::{collections::BTreeMap, fmt, sync::Arc};

use ::value::Value;
use vrl::state::TypeState;
use vrl::{
    diagnostic::{Label, Span},
    prelude::*,
};

static MEZMO_PATTERNS: &[(&str, &str)] = &[
    ("JSON_OBJECT", r#"{.*}"#),
    ("JSON_ARRAY", r#"\[.*\]"#),
    ("XML", r#"<[\w"=\s]+>.*<\/[\w\s]+>"#),
    ("CURLY_BRACKET", r#"{|}"#),
    ("SQUARE_BRACKET", r#"\[|\]"#),
    ("TAB", r#"\t"#),
    ("SINGLE_SPACE", r#"\s"#), // SPACE is already defined in builtin patterns but it's greedy
    ("ANY_CHAR", r#"."#),
    ("DOUBLE_QUOTE", r#"""#),
    ("SINGLE_QUOTE", r#"'"#),
    ("PERIOD", r#"\."#),
    ("UTF8_WORD", r#"\p{L}+"#),
    ("SENTENCE", r#"[\p{L},":;\s\-]+"#), // Sentence with UTF8 chars
    ("OPTIONAL_SENTENCE", r#"[\p{L},":;\s\-]*"#),
];

fn parse_grok(value: Value, pattern: Arc<grok::Pattern>) -> Resolved {
    let bytes = value.try_bytes_utf8_lossy()?;
    match pattern.match_against(&bytes) {
        Some(matches) => {
            let mut result = BTreeMap::new();

            for (name, value) in matches.iter() {
                result.insert(name.to_string(), Value::from(value));
            }

            Ok(Value::from(result))
        }
        None => Err("unable to parse input with grok pattern".into()),
    }
}

#[derive(Debug)]
pub(crate) enum Error {
    InvalidGrokPattern(grok::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidGrokPattern(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl DiagnosticMessage for Error {
    fn code(&self) -> usize {
        109
    }

    fn labels(&self) -> Vec<Label> {
        match self {
            Error::InvalidGrokPattern(err) => {
                vec![Label::primary(
                    format!("grok pattern error: {err}"),
                    Span::default(),
                )]
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ParseGrok;

impl Function for ParseGrok {
    fn identifier(&self) -> &'static str {
        "parse_grok"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[
            Parameter {
                keyword: "value",
                kind: kind::BYTES,
                required: true,
            },
            Parameter {
                keyword: "pattern",
                kind: kind::BYTES,
                required: true,
            },
        ]
    }

    fn examples(&self) -> &'static [Example] {
        &[Example {
            title: "parse grok pattern",
            source: indoc! {r#"
                value = "2020-10-02T23:22:12.223222Z info Hello world"
                pattern = "%{TIMESTAMP_ISO8601:timestamp} %{LOGLEVEL:level} %{GREEDYDATA:message}"

                parse_grok!(value, pattern)
            "#},
            result: Ok(indoc! {r#"
                {
                    "timestamp": "2020-10-02T23:22:12.223222Z",
                    "level": "info",
                    "message": "Hello world"
                }
            "#}),
        }]
    }

    fn compile(
        &self,
        _state: &state::TypeState,
        _ctx: &mut FunctionCompileContext,
        arguments: ArgumentList,
    ) -> Compiled {
        let value = arguments.required("value");

        let pattern = arguments
            .required_literal("pattern")?
            .to_value()
            .try_bytes_utf8_lossy()
            .expect("grok pattern not bytes")
            .into_owned();

        let mut grok = grok_with_mezmo_patterns();
        let pattern =
            Arc::new(grok.compile(&pattern, true).map_err(|e| {
                Box::new(Error::InvalidGrokPattern(e)) as Box<dyn DiagnosticMessage>
            })?);

        Ok(ParseGrokFn { value, pattern }.as_expr())
    }
}

fn grok_with_mezmo_patterns() -> grok::Grok {
    let mut grok = grok::Grok::with_default_patterns();
    for &(key, value) in MEZMO_PATTERNS {
        grok.add_pattern(String::from(key), String::from(value));
    }
    grok
}

#[derive(Clone, Debug)]
struct ParseGrokFn {
    value: Box<dyn Expression>,

    // Wrapping pattern in an Arc, as cloning the pattern could otherwise be expensive.
    pattern: Arc<grok::Pattern>,
}

impl FunctionExpression for ParseGrokFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        let value = self.value.resolve(ctx)?;
        let pattern = self.pattern.clone();

        parse_grok(value, pattern)
    }

    fn type_def(&self, _: &TypeState) -> TypeDef {
        TypeDef::object(Collection::any()).fallible()
    }
}

#[cfg(test)]
mod test {
    use vector_common::btreemap;

    use super::*;

    test_function![
        parse_grok => ParseGrok;

        invalid_grok {
            args: func_args![ value: "foo",
                              pattern: "%{NOG}"],
            want: Err("The given pattern definition name \"NOG\" could not be found in the definition map"),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        error {
            args: func_args![ value: "an ungrokkable message",
                              pattern: "%{TIMESTAMP_ISO8601:timestamp} %{LOGLEVEL:level} %{GREEDYDATA:message}"],
            want: Err("unable to parse input with grok pattern"),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        error2 {
            args: func_args![ value: "2020-10-02T23:22:12.223222Z an ungrokkable message",
                              pattern: "%{TIMESTAMP_ISO8601:timestamp} %{LOGLEVEL:level} %{GREEDYDATA:message}"],
            want: Err("unable to parse input with grok pattern"),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed {
            args: func_args![ value: "2020-10-02T23:22:12.223222Z info Hello world",
                              pattern: "%{TIMESTAMP_ISO8601:timestamp} %{LOGLEVEL:level} %{GREEDYDATA:message}"],
            want: Ok(Value::from(btreemap! {
                "timestamp" => "2020-10-02T23:22:12.223222Z",
                "level" => "info",
                "message" => "Hello world",
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed2 {
            args: func_args![ value: "2020-10-02T23:22:12.223222Z",
                              pattern: "(%{TIMESTAMP_ISO8601:timestamp}|%{LOGLEVEL:level})"],
            want: Ok(Value::from(btreemap! {
                "timestamp" => "2020-10-02T23:22:12.223222Z",
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_json_object {
            args: func_args![ value: r#"hello {"prop": 1} info"#,
                            pattern: "%{DATA}%{JSON_OBJECT:json} %{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "json" => r#"{"prop": 1}"#,
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_json_array {
            args: func_args![ value: "hello [1, 2, 3] info",
                            pattern: "%{DATA}%{JSON_ARRAY:json} %{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "json" => "[1, 2, 3]",
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_xml {
            args: func_args![ value: r#"hello <tag prop="value">inner</tag> info"#,
                            pattern: "%{DATA}%{XML:xml} %{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "xml" => r#"<tag prop="value">inner</tag>"#,
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_curly_bracket {
            args: func_args![ value: "hello {world} info",
                            pattern: "hello %{CURLY_BRACKET}%{DATA:body}%{CURLY_BRACKET} %{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "body" => "world",
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_square_bracket {
            args: func_args![ value: "hello [world] info",
                            pattern: "hello %{SQUARE_BRACKET}%{DATA:body}%{SQUARE_BRACKET} %{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "body" => "world",
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_any_char {
            args: func_args![ value: "hello `world* info",
                            pattern: "hello %{ANY_CHAR}%{WORD:body}%{ANY_CHAR} %{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "body" => "world",
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_tab {
            args: func_args![value: "hello	world	info", // separated by tabs
                            pattern: "hello%{TAB}%{WORD:body}%{TAB}%{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "body" => "world",
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_single_space {
            args: func_args![ value: "hello world info",
                            pattern: "hello%{SINGLE_SPACE}%{WORD:body}%{SINGLE_SPACE}%{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "body" => "world",
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_double_quote {
            args: func_args![ value: "hello \"world\" info",
                            pattern: "hello %{DOUBLE_QUOTE}%{WORD:body}%{DOUBLE_QUOTE} %{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "body" => "world",
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_single_quote {
            args: func_args![ value: "hello 'world' info",
                            pattern: "hello %{SINGLE_QUOTE}%{WORD:body}%{SINGLE_QUOTE} %{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "body" => "world",
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_period {
            args: func_args![ value: "hello .world info",
                            pattern: "hello %{PERIOD}%{WORD:body} %{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "body" => "world",
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_utf8_word {
            args: func_args![ value: "hello Visca Barça España 你好 info",
                            pattern: "hello %{UTF8_WORD:a} %{UTF8_WORD:b} %{UTF8_WORD:c} %{UTF8_WORD:d} %{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "a" => "Visca",
                "b" => "Barça",
                "c" => "España",
                "d" => "你好",
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_sentence {
            args: func_args![ value: "hello Visca Barça España 你好 info",
                            pattern: "hello %{SENTENCE:body} %{LOGLEVEL:level}"],
            want: Ok(Value::from(btreemap! {
                "body" => "Visca Barça España 你好",
                "level" => "info"
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }

        parsed_mezmo_optional_sentence {
            args: func_args![ value: "hello Visca Barça España 你好 info",
                            pattern: "hello %{OPTIONAL_SENTENCE:body} %{LOGLEVEL:level}%{OPTIONAL_SENTENCE:empty}"],
            want: Ok(Value::from(btreemap! {
                "body" => "Visca Barça España 你好",
                "level" => "info",
                "empty" => ""
            })),
            tdef: TypeDef::object(Collection::any()).fallible(),
        }
    ];
}
