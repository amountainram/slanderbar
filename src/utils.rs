use std::fmt::Error;

use nom::{
    bytes::{complete::take_until, streaming::tag},
    combinator::map_res,
    sequence::delimited,
    IResult,
};

#[derive(Debug)]
pub struct TaggedTemplate<'a> {
    literals: Vec<&'a str>,
    variables: Vec<&'a str>,
}

impl<'a> TaggedTemplate<'a> {
    pub fn new() -> Self {
        Self {
            literals: vec![],
            variables: vec![],
        }
    }
}

pub fn parser(text: &str) -> IResult<&str, TaggedTemplate> {
    let mut next_part = text;
    if next_part.len() == 0 {
        return Ok((
            "",
            TaggedTemplate {
                literals: vec![""],
                variables: vec![],
            },
        ));
    }

    let mut tagged_template = TaggedTemplate::new();
    loop {
        let (rest, found) = take_until("{{")(next_part)
            .unwrap_or_else(|_: nom::Err<nom::error::Error<_>>| (next_part, ""));
        let result: IResult<&str, &str> = map_res(
            delimited(tag("{{"), take_until("}}"), tag("}}")),
            |key: &str| Ok::<&str, Error>(key.trim()),
        )(rest);

        next_part = match result {
            Ok((rest, var)) => {
                tagged_template.literals.push(found);
                tagged_template.variables.push(var);
                rest
            }
            Err(_) => {
                tagged_template.literals.push(next_part);
                ""
            }
        };

        if next_part.len() == 0 {
            return Ok((next_part, tagged_template));
        }
    }
}

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_cases() {
        let (_, template) = parser("").unwrap();
        assert_eq!(template.literals.len(), 1);
        assert_eq!(*template.literals.get(0).unwrap(), "");

        assert_eq!(template.variables.len(), 0);
    }

    #[test]
    fn should_parse_a_basic_string() {
        let (_, template) = parser(r#"hello {{ world }}"#).unwrap();
        assert_eq!(template.literals.len(), 1);
        assert_eq!(*template.literals.get(0).unwrap(), "hello ");

        assert_eq!(template.variables.len(), 1);
        assert_eq!(*template.variables.get(0).unwrap(), "world");
    }

    #[test]
    fn should_parse_a_basic_string_and_continue() {
        let (_, template) = parser(r#"hello {{ world }}!"#).unwrap();
        assert_eq!(template.literals.len(), 2);
        assert_eq!(*template.literals.get(0).unwrap(), "hello ");
        assert_eq!(*template.literals.get(1).unwrap(), "!");

        assert_eq!(template.variables.len(), 1);
        assert_eq!(*template.variables.get(0).unwrap(), "world");
    }

    #[test]
    fn should_parse_a_string_that_ends_with_incomplete_variable() {
        let (_, template) = parser(r#"hello {{ world }} and also to {{ you"#).unwrap();
        assert_eq!(template.literals.len(), 2);
        assert_eq!(*template.literals.get(0).unwrap(), "hello ");
        assert_eq!(*template.literals.get(1).unwrap(), " and also to {{ you");

        assert_eq!(template.variables.len(), 1);
        assert_eq!(*template.variables.get(0).unwrap(), "world");
    }
}
