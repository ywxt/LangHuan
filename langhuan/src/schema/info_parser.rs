use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::{line_ending, not_line_ending, space0},
    Finish, IResult,
};

use crate::Result;

fn match_allowed_name(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '.' || c == '-')(input)
}

fn parse_field_name(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("--")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag("@")(input)?;
    let (input, name) = match_allowed_name(input)?;
    Ok((input, name))
}

fn parse_field_value(input: &str) -> IResult<&str, &str> {
    let (input, value) = not_line_ending(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, value.trim()))
}

fn parse_field(input: &str) -> IResult<&str, (&str, &str)> {
    let (input, name) = parse_field_name(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = space0(input)?;
    let (input, value) = parse_field_value(input)?;
    Ok((input, (name, value)))
}


fn parse_whitespace_line(input: &str) -> IResult<&str, ()> {
    let (input, _) = space0(input)?;
    let (input, _) = line_ending(input)?;
    Ok((input, ()))
}

fn parse_line(input: &str) -> IResult<&str, Line> {
    if let Ok((input, _)) = parse_whitespace_line(input) {
        return Ok((input, Line::Whitespace));
    }
    let (input, (name, value)) = parse_field(input)?;
    Ok((input, Line::Field(Field { name, value })))
}

#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Debug, PartialEq)]
pub enum Line<'a> {
    Field(Field<'a>),
    Whitespace,
}

pub struct FieldIter<'a> {
    input: &'a str,
}
impl<'a> Iterator for FieldIter<'a> {
    type Item = Result<Field<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }
        let (new_input, line) = match parse_line(self.input)
            .finish()
            .map_err(|e| crate::Error::ScriptParseError(format!("{}", e)))
        {
            Ok(result) => result,
            Err(e) => return Some(Err(e)),
        };
        let result = match line {
            Line::Field(field) => Some(Ok(field)),
            Line::Whitespace => None,
        };
        self.input = new_input;
        result
    }
}
pub fn parse_script(input: &'_ str) -> FieldIter<'_> {
    FieldIter { input }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field_name() {
        let input = "--@name:";
        let (input, output) = parse_field_name(input).unwrap();
        assert_eq!(input, ":");
        assert_eq!(output, "name");
        let input = "--  @name: value";
        let (input, output) = parse_field_name(input).unwrap();
        assert_eq!(input, ": value");
        assert_eq!(output, "name");

        let input = "--@name_1: value";
        let (_, output) = parse_field_name(input).unwrap();
        assert_eq!(output, "name_1");
        let input = "--@name.1: value";
        let (_, output) = parse_field_name(input).unwrap();
        assert_eq!(output, "name.1");
        let input = "--@name-1: value";
        let (_, output) = parse_field_name(input).unwrap();
        assert_eq!(output, "name-1");
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
                name: "name",
                value: "value"
            })
        );

        let input = "--@name: value";
        let output = parse_line(input);
        assert!(output.is_err());
    }

    #[test]
    fn test_parse_script() {
        let input = r#"--@name: value
--@name_2: value2
--@name.3: 1.0
"#;
        let output: Vec<Field> = parse_script(input).collect::<Result<_>>().unwrap();
        assert_eq!(
            output,
            vec![
                Field {
                    name: "name",
                    value: "value"
                },
                Field {
                    name: "name_2",
                    value: "value2"
                },
                Field {
                    name: "name.3",
                    value: "1.0"
                }
            ]
        );
    }
}
