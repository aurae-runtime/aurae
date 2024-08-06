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
use std::fmt::Display;
use validator::validate_range;

pub fn minimum_value<T: PartialOrd + PartialEq + Display + Copy>(
    value: T,
    minimum: T,
    units: &str,
    field_name: &str,
    parent_name: Option<&str>,
) -> Result<(), ValidationError> {
    match validate_range(value, Some(minimum), None) {
        true => Ok(()),
        false => Err(ValidationError::Minimum {
            field: super::field_name(field_name, parent_name),
            minimum: minimum.to_string(),
            units: units.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimum_value() {
        assert!(matches!(
            minimum_value(1, 2, "test", "test", None),
            Err(ValidationError::Minimum { .. })
        ));

        assert!(matches!(minimum_value(2, 1, "test", "test", None), Ok(..)));
    }
}