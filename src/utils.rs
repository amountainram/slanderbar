use std::fmt::Error;

use nom::{
    bytes::complete::{tag, take_until},
    combinator::map_res,
    sequence::delimited,
    IResult,
};

#[derive(Debug)]
pub struct TaggedTemplate<'a> {
    pub literals: Vec<&'a str>,
    pub variables: Vec<&'a str>,
}

impl<'a> TaggedTemplate<'a> {
    pub fn new() -> Self {
        Self {
            literals: vec![],
            variables: vec![],
        }
    }
}

fn remove_braces(input: &str) -> &str {
    let (_, found) = map_res(
        delimited(tag("["), take_until("]"), tag("]")),
        |wrapped_key: &str| {
            let trimmed_key = wrapped_key.trim();

            if trimmed_key.len() == 0 {
                return Ok::<&str, Error>("");
            }

            let real_key = match (&trimmed_key[0..1], &trimmed_key[trimmed_key.len() - 1..]) {
                (r#"""#, r#"""#) | ("'", "'") => &trimmed_key[1..trimmed_key.len() - 1],
                _ => trimmed_key,
            };

            Ok::<&str, Error>(real_key)
        },
    )(input)
    .unwrap_or_else(|_: nom::Err<nom::error::Error<_>>| ("", input.trim()));

    found
}

pub fn key_parser(text: &str) -> Vec<&str> {
    let mut next_part = text;
    if next_part.len() == 0 {
        return vec![];
    }

    let mut keys = vec![];
    loop {
        match take_until::<&str, &str, nom::error::Error<_>>(".")(next_part) {
            Ok((rest, found)) => {
                keys.push(remove_braces(found));
                next_part = &rest[1..];
            }
            Err(_) => {
                keys.push(remove_braces(next_part));
                return keys;
            }
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

                if rest.len() == 0 {
                    tagged_template.literals.push("");
                }

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
        assert_eq!(template.literals.len(), 2);
        assert_eq!(*template.literals.get(0).unwrap(), "hello ");
        assert_eq!(*template.literals.get(1).unwrap(), "");

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

    #[test]
    fn should_return_more_literals_than_variables_with_an_excess_of_one() {
        let (_, template) = parser(r#"hello {{ world }} and also to {{ you }}"#).unwrap();
        assert_eq!(template.literals.len(), 3);
        assert_eq!(*template.literals.get(0).unwrap(), "hello ");
        assert_eq!(*template.literals.get(1).unwrap(), " and also to ");
        assert_eq!(*template.literals.get(2).unwrap(), "");

        assert_eq!(template.variables.len(), 2);
        assert_eq!(*template.variables.get(0).unwrap(), "world");
        assert_eq!(*template.variables.get(1).unwrap(), "you");
    }

    #[test]
    fn should_remove_braces_to_key() {
        let keys = key_parser("first.['a key'].  another_key.[   '0' ].[         its me ].wei[rdo");

        vec!["first", "a key", "another_key", "0", "its me", "wei[rdo"]
            .iter()
            .zip(&keys)
            .for_each(|(&a, &b)| {
                assert_eq!(a, b);
            });
    }
}
