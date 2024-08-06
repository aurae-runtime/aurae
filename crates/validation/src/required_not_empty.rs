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
#[cfg(feature = "secrecy")]
use secrecy::{ExposeSecret, SecretString};
use validator::HasLen;

pub fn required_not_empty<T: HasLen>(
    value: Option<T>,
    field_name: &str,
    parent_name: Option<&str>,
) -> Result<T, ValidationError> {
    let value = super::required(value, field_name, parent_name)?;

    if value.length() == 0 {
        return Err(ValidationError::Required {
            field: super::field_name(field_name, parent_name),
        });
    }

    Ok(value)
}

#[cfg(feature = "secrecy")]
pub fn required_not_empty_secret_string(
    value: Option<SecretString>,
    field_name: &str,
    parent_name: Option<&str>,
) -> Result<SecretString, ValidationError> {
    let value = super::required(value, field_name, parent_name)?;

    if value.expose_secret().is_empty() {
        return Err(ValidationError::Required {
            field: super::field_name(field_name, parent_name),
        });
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_not_empty() {
        assert!(matches!(
            required_not_empty(Some("hi"), "test", None),
            Ok(x) if x == "hi"
        ));

        assert!(matches!(
            required_not_empty(None::<String>, "test", None),
            Err(ValidationError::Required { .. })
        ));

        assert!(matches!(
            required_not_empty(Some(""), "test", None),
            Err(ValidationError::Required { .. })
        ));
    }

    #[cfg(feature = "secrecy")]
    #[test]
    fn test_required_not_empty_secret_string() {
        assert!(matches!(
            required_not_empty_secret_string(Some(SecretString::new("hi".to_string())), "test", None),
            Ok(x) if x.expose_secret() == "hi"
        ));

        assert!(matches!(
            required_not_empty_secret_string(None, "test", None),
            Err(ValidationError::Required { .. })
        ));

        assert!(matches!(
            required_not_empty_secret_string(
                Some(SecretString::new("".to_string())),
                "test",
                None
            ),
            Err(ValidationError::Required { .. })
        ));
    }
}