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
use validator::{validate_length, HasLen};

pub fn maximum_length<T: HasLen>(
    value: T,
    length: u64,
    units: &str,
    field_name: &str,
    parent_name: Option<&str>,
) -> Result<(), ValidationError> {
    match validate_length(value, None, Some(length), None) {
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
    use validator::HasLen;

    #[test]
    fn test_maximum_length() {
        let value = vec![1, 2];

        let maximum = value.length() - 1;
        let result = maximum_length(&value, maximum, "test", "test", None);
        assert!(matches!(result, Err(ValidationError::Maximum { .. })));

        let maximum = value.length();
        let result = maximum_length(&value, maximum, "test", "test", None);
        assert!(matches!(result, Ok(..)));
    }
}
