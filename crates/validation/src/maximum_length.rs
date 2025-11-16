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
use validator::ValidateLength;

pub fn maximum_length<T>(
    value: T,
    length: u64,
    units: &str,
    field_name: &str,
    parent_name: Option<&str>,
) -> Result<(), ValidationError>
where
    T: ValidateLength<u64>,
{
    match value.validate_length(None, Some(length), None) {
        true => Ok(()),
        false => Err(ValidationError::Maximum {
            field: super::field_name(field_name, parent_name),
            maximum: length.to_string(),
            units: units.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::ValidateLength;

    #[test]
    fn test_maximum_length() {
        let value = vec![1, 2];
        let current_length = value.length().expect("vector always has length");

        let maximum = current_length - 1;
        let result = maximum_length(&value, maximum, "test", "test", None);
        assert!(matches!(result, Err(ValidationError::Maximum { .. })));

        let maximum = current_length;
        let result = maximum_length(&value, maximum, "test", "test", None);
        assert!(matches!(result, Ok(..)));
    }
}
