use nu_cmd_base::input_handler::{operate, CmdArgument};
use nu_engine::CallExt;
use nu_protocol::{
    ast::{Call, CellPath},
    engine::{Command, EngineState, Stack},
    Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, Span, SyntaxShape,
    Type, Value,
};
use num_traits::ToPrimitive;

pub struct Arguments {
    cell_paths: Option<Vec<CellPath>>,
}

impl CmdArgument for Arguments {
    fn take_cell_paths(&mut self) -> Option<Vec<CellPath>> {
        self.cell_paths.take()
    }
}

#[derive(Clone)]
pub struct BitsInto;

impl Command for BitsInto {
    fn name(&self) -> &str {
        "into bits"
    }

    fn signature(&self) -> Signature {
        Signature::build("into bits")
            .input_output_types(vec![
                (Type::Binary, Type::String),
                (Type::Int, Type::String),
                (Type::Filesize, Type::String),
                (Type::Duration, Type::String),
                (Type::String, Type::String),
                (Type::Bool, Type::String),
                (Type::Date, Type::String),
                (Type::Table(vec![]), Type::Table(vec![])),
                (Type::Record(vec![]), Type::Record(vec![])),
            ])
            .allow_variants_without_examples(true) // TODO: supply exhaustive examples
            .rest(
                "rest",
                SyntaxShape::CellPath,
                "for a data structure input, convert data at the given cell paths",
            )
            .category(Category::Conversions)
    }

    fn usage(&self) -> &str {
        "Convert value to a binary primitive."
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["convert", "cast"]
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        into_bits(engine_state, stack, call, input)
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "convert a binary value into a string, padded to 8 places with 0s",
                example: "01b | into bits",
                result: Some(Value::string("00000001",
                    Span::test_data(),
                )),
            },
            Example {
                description: "convert an int into a string, padded to 8 places with 0s",
                example: "1 | into bits",
                result: Some(Value::string("00000001",
                    Span::test_data(),
                )),
            },
            Example {
                description: "convert a filesize value into a string, padded to 8 places with 0s",
                example: "1b | into bits",
                result: Some(Value::string("00000001",
                    Span::test_data(),
                )),
            },
            Example {
                description: "convert a duration value into a string, padded to 8 places with 0s",
                example: "1ns | into bits",
                result: Some(Value::string("00000001",
                    Span::test_data(),
                )),
            },
            Example {
                description: "convert a boolean value into a string, padded to 8 places with 0s",
                example: "true | into bits",
                result: Some(Value::string("00000001",
                    Span::test_data(),
                )),
            },
            Example {
                description: "convert a datetime value into a string, padded to 8 places with 0s",
                example: "2023-04-17T01:02:03 | into bits",
                result: Some(Value::string("01001101 01101111 01101110 00100000 01000001 01110000 01110010 00100000 00110001 00110111 00100000 00110000 00110001 00111010 00110000 00110010 00111010 00110000 00110011 00100000 00110010 00110000 00110010 00110011",
                    Span::test_data(),
                )),
            },
            Example {
                description: "convert a string into a raw binary string, padded with 0s to 8 places",
                example: "'nushell.sh' | into bits",
                result: Some(Value::string("01101110 01110101 01110011 01101000 01100101 01101100 01101100 00101110 01110011 01101000",
                    Span::test_data(),
                )),
            },
        ]
    }
}

fn into_bits(
    engine_state: &EngineState,
    stack: &mut Stack,
    call: &Call,
    input: PipelineData,
) -> Result<PipelineData, ShellError> {
    let head = call.head;
    let cell_paths = call.rest(engine_state, stack, 0)?;
    let cell_paths = (!cell_paths.is_empty()).then_some(cell_paths);

    match input {
        PipelineData::ExternalStream { stdout: None, .. } => {
            Ok(Value::binary(vec![], head).into_pipeline_data())
        }
        PipelineData::ExternalStream {
            stdout: Some(stream),
            ..
        } => {
            // TODO: in the future, we may want this to stream out, converting each to bytes
            let output = stream.into_bytes()?;
            Ok(Value::binary(output.item, head).into_pipeline_data())
        }
        _ => {
            let args = Arguments { cell_paths };
            operate(action, args, input, call.head, engine_state.ctrlc.clone())
        }
    }
}

fn convert_to_smallest_number_type(num: i64, span: Span) -> Value {
    if let Some(v) = num.to_i8() {
        let bytes = v.to_ne_bytes();
        let mut raw_string = "".to_string();
        for ch in bytes {
            raw_string.push_str(&format!("{:08b} ", ch));
        }
        Value::string(raw_string.trim(), span)
    } else if let Some(v) = num.to_i16() {
        let bytes = v.to_ne_bytes();
        let mut raw_string = "".to_string();
        for ch in bytes {
            raw_string.push_str(&format!("{:08b} ", ch));
        }
        Value::string(raw_string.trim(), span)
    } else if let Some(v) = num.to_i32() {
        let bytes = v.to_ne_bytes();
        let mut raw_string = "".to_string();
        for ch in bytes {
            raw_string.push_str(&format!("{:08b} ", ch));
        }
        Value::string(raw_string.trim(), span)
    } else {
        let bytes = num.to_ne_bytes();
        let mut raw_string = "".to_string();
        for ch in bytes {
            raw_string.push_str(&format!("{:08b} ", ch));
        }
        Value::string(raw_string.trim(), span)
    }
}

pub fn action(input: &Value, _args: &Arguments, span: Span) -> Value {
    match input {
        Value::Binary { val, .. } => {
            let mut raw_string = "".to_string();
            for ch in val {
                raw_string.push_str(&format!("{:08b} ", ch));
            }
            Value::string(raw_string.trim(), span)
        }
        Value::Int { val, .. } => convert_to_smallest_number_type(*val, span),
        Value::Filesize { val, .. } => convert_to_smallest_number_type(*val, span),
        Value::Duration { val, .. } => convert_to_smallest_number_type(*val, span),
        Value::String { val, .. } => {
            let raw_bytes = val.as_bytes();
            let mut raw_string = "".to_string();
            for ch in raw_bytes {
                raw_string.push_str(&format!("{:08b} ", ch));
            }
            Value::string(raw_string.trim(), span)
        }
        Value::Bool { val, .. } => {
            let v = <i64 as From<bool>>::from(*val);
            convert_to_smallest_number_type(v, span)
        }
        Value::Date { val, .. } => {
            let value = val.format("%c").to_string();
            let bytes = value.as_bytes();
            let mut raw_string = "".to_string();
            for ch in bytes {
                raw_string.push_str(&format!("{:08b} ", ch));
            }
            Value::string(raw_string.trim(), span)
        }
        // Propagate errors by explicitly matching them before the final case.
        Value::Error { .. } => input.clone(),
        other => Value::error(
            ShellError::OnlySupportsThisInputType {
                exp_input_type: "integer, filesize, string, date, duration, binary or bool".into(),
                wrong_type: other.get_type().to_string(),
                dst_span: span,
                src_span: other.span(),
            },
            span,
        ),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(BitsInto {})
    }
}
