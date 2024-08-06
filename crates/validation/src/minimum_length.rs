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
use validator::{validate_length, HasLen};

pub fn minimum_length<T: HasLen>(
    value: T,
    length: u64,
    units: &str,
    field_name: &str,
    parent_name: Option<&str>,
) -> Result<(), ValidationError> {
    match validate_length(value, Some(length), None, None) {
        true => Ok(()),
        false => Err(ValidationError::Minimum {
            field: super::field_name(field_name, parent_name),
            minimum: length.to_string(),
            units: units.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::HasLen;

    #[test]
    fn test_minimum_length() {
        let value = "123456";

        assert!(matches!(
            minimum_length(value, value.length() + 1, "test", "test", None),
            Err(ValidationError::Minimum { .. })
        ));

        assert!(matches!(
            minimum_length(value, value.length() - 1, "test", "test", None),
            Ok(..)
        ));
    }
}