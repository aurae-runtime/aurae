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

pub fn required<T>(
    value: Option<T>,
    field_name: &str,
    parent_name: Option<&str>,
) -> Result<T, ValidationError> {
    match value {
        None => Err(ValidationError::Required {
            field: super::field_name(field_name, parent_name),
        }),
        Some(value) => Ok(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required() {
        assert!(matches!(
            required(Some("hi"), "test", None),
            Ok(x) if x == "hi"
        ));

        assert!(matches!(
            required(None::<String>, "test", None),
            Err(ValidationError::Required { .. })
        ));
    }
}