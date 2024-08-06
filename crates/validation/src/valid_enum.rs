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

pub fn valid_enum<T: TryFrom<i32>>(
    value: i32,
    field_name: &str,
    parent_name: Option<&str>,
) -> Result<T, ValidationError> {
    match T::try_from(value) {
        Ok(value) => Ok(value),
        Err(_) => Err(ValidationError::Invalid {
            field: super::field_name(field_name, parent_name),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_enum_derive::TryFromPrimitive;

    #[derive(TryFromPrimitive)]
    #[repr(i32)]
    enum MyTestEnum {
        First = 10,
        Second = 11,
    }

    #[test]
    fn test_valid_enum() {
        assert!(matches!(valid_enum(10, "test", None), Ok(MyTestEnum::First)));

        assert!(matches!(
            valid_enum::<MyTestEnum>(12312, "test", None),
            Err(ValidationError::Invalid { .. })
        ));
    }
}