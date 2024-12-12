use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::{alphanumeric1, line_ending, space0},
    combinator::map,
    error::{convert_error, VerboseError},
    sequence::{terminated, tuple},
    Finish, IResult,
};

use crate::Result;

fn parse_field_name(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    map(
        tuple((
            tag("--"),
            space0,
            tag("@"),
            terminated(alphanumeric1, tag(":")),
        )),
        |(_, _, _, name)| name,
    )(input)
}

fn parse_field_value(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    map(
        terminated(take_while1(|c: char| c != '\n' && c != '\r'), line_ending),
        |value: &str| value.trim(),
    )(input)
}

fn parse_field(input: &str) -> IResult<&str, (&str, &str), VerboseError<&str>> {
    map(
        tuple((parse_field_name, space0, parse_field_value)),
        |(name, _, value)| (name, value),
    )(input)
}

fn parse_whitespace_line(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    terminated(space0, line_ending)(input)
}

#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub value: String,
}

#[derive(Debug, PartialEq)]
enum Line {
    Field(Field),
    Whitespace,
}

fn parse_line(input: &str) -> IResult<&str, Line, VerboseError<&str>> {
    if let Ok((input, _)) = parse_whitespace_line(input) {
        return Ok((input, Line::Whitespace));
    }
    let (input, (name, value)) = parse_field(input)?;
    let name = name.to_string();
    let value = value.to_string();
    Ok((input, Line::Field(Field { name, value })))
}

pub fn parse_script(mut input: &str) -> Result<Vec<Field>> {
    let mut fields = Vec::new();
    while !input.is_empty() {
        let (new_input, line) = parse_line(input)
            .finish()
            .map_err(|e| crate::Error::ParseError(convert_error(input, e)))?;
        match line {
            Line::Field(field) => fields.push(field),
            Line::Whitespace => break,
        }
        input = new_input;
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_name() {
        let input = "--@name:";
        let (input, output) = parse_field_name(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(output, "name");
        let input = "--  @name: value";
        let (input, output) = parse_field_name(input).unwrap();
        assert_eq!(input, " value");
        assert_eq!(output, "name");
    }

    #[test]
    fn test_parse_field_value() {
        let input = "value    \n";
        let (input, output) = parse_field_value(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(output, "value");
    }

    #[test]
    fn test_parse_field() {
        let input = "--@name: value\n";
        let (input, output) = parse_field(input).unwrap();
        assert_eq!(input, "");
        assert_eq!(output, ("name", "value"));
    }

    #[test]
    fn test_parse_whitespace_line() {
        let input = " \n";
        let (input, _) = parse_whitespace_line(input).unwrap();
        assert_eq!(input, "");

        let input = " \r\n";
        let (input, _) = parse_whitespace_line(input).unwrap();
        assert_eq!(input, "");

        let input = "testdata\n";
        let result = parse_whitespace_line(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_line() {
        let input = " \n";
        let (_, output) = parse_line(input).unwrap();
        assert_eq!(output, Line::Whitespace);

        let input = "--  @name: value   \n";
        let (_, output) = parse_line(input).unwrap();
        assert_eq!(
            output,
            Line::Field(Field {
                name: "name".to_string(),
                value: "value".to_string()
            })
        );

        let input = "--@name: value";
        let output = parse_line(input);
        assert!(output.is_err());
    }
}
