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
use super::ValidationError;
use fancy_regex::Regex;

pub fn allow_regex(
    value: &str,
    pattern: &Regex,
    field_name: &str,
    parent_name: Option<&str>,
) -> Result<(), ValidationError> {
    match pattern.is_match(value) {
        Ok(true) => Ok(()),
        Ok(false) | Err(_) => Err(ValidationError::AllowRegexViolation {
            field: super::field_name(field_name, parent_name),
            pattern: pattern.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DOMAIN_NAME_LABEL_REGEX;

    #[test]
    fn test_allow_regex() {
        assert!(matches!(
            allow_regex("my-name", &DOMAIN_NAME_LABEL_REGEX, "test", None),
            Ok(..)
        ));

        assert!(matches!(
            allow_regex("my*name", &DOMAIN_NAME_LABEL_REGEX, "test", None),
            Err(ValidationError::AllowRegexViolation { .. })
        ));
    }
}