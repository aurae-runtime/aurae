/* -------------------------------------------------------------------------- *\
 *               Apache 2.0 License Copyright The Aurae Authors               *
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
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
