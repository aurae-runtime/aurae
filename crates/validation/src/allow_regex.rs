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
