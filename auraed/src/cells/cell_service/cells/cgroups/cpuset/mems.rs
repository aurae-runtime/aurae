/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
\* -------------------------------------------------------------------------- */
use fancy_regex::Regex;
use lazy_static::lazy_static;
use std::{
    fmt::{Display, Formatter},
    ops::Deref,
};
use validation::{ValidatedField, ValidationError};

lazy_static! {
    // input should be a comma seperated list of numbers with optional -s
    // or the empty string.
    static ref MEMS_INPUT_REGEX:  Regex = {
            Regex::new(r"^(\d(-\d)?,?)*$").expect("regex construction")
    };
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Mems(String);

impl Mems {
    #[cfg(test)]
    pub fn new(cpu_cpus: String) -> Self {
        Self(cpu_cpus)
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl ValidatedField<String> for Mems {
    fn validate(
        input: Option<String>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input = validation::required(input, field_name, parent_name)?;

        validation::allow_regex(
            &input,
            &MEMS_INPUT_REGEX,
            field_name,
            parent_name,
        )?;

        Ok(Self(input))
    }
}

impl Deref for Mems {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Mems {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_test_case::test_case;

    #[test_case(""; "empty string")]
    #[test_case("0"; "just one cpu")]
    #[test_case("1,2"; "comma seperation")]
    #[test_case("1-3"; "a range")]
    #[test_case("1,2-5,6"; "combo")]
    #[test]
    fn test_validation_success(input: &str) {
        assert!(
            Mems::validate(Some(input.to_string()), "cpu_cpus", None).is_ok()
        );
    }

    #[test_case("foo"; "text")]
    #[test_case("1:2"; "colon seperation")]
    #[test_case("1..3"; "not a range")]
    #[test_case("1,foo;5"; "bad combo")]
    #[test]
    fn test_validation_failure(input: &str) {
        assert!(
            Mems::validate(Some(input.to_string()), "cpu_cpus", None).is_err()
        );
    }
}