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
use validator::ValidateRange;

pub fn maximum_value<T>(
    value: T,
    maximum: T,
    units: &str,
    field_name: &str,
    parent_name: Option<&str>,
) -> Result<(), ValidationError>
where
    T: ValidateRange<T> + PartialOrd + PartialEq + Display + Copy,
{
    match value.validate_range(None, Some(maximum), None, None) {
        true => Ok(()),
        false => Err(ValidationError::Maximum {
            field: super::field_name(field_name, parent_name),
            maximum: maximum.to_string(),
            units: units.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maximum_value() {
        assert!(matches!(maximum_value(1, 2, "test", "test", None), Ok(..)));

        assert!(matches!(
            maximum_value(2, 1, "test", "test", None),
            Err(ValidationError::Maximum { .. })
        ));
    }
}
