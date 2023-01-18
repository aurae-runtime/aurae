/* -------------------------------------------------------------------------- *\
#             Apache 2.0 License Copyright © The Aurae Authors                #
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

use crate::runtime::cell_service::cells::CellName;
use iter_tools::Itertools;
use std::collections::VecDeque;
use validation::{ValidatedField, ValidationError};

pub const SEPARATOR: &str = "/";

#[derive(Debug, Clone)]
pub enum CellNamePath {
    Empty,
    CellName(CellName),
    Path(VecDeque<CellName>),
}

impl CellNamePath {
    /// Returns [None] if current variant is [CellNamePath::Empty]
    pub fn into_child(self) -> Option<(CellName, Self)> {
        match self {
            CellNamePath::Empty => None,
            CellNamePath::CellName(cell_name) => Some((cell_name, Self::Empty)),
            CellNamePath::Path(mut parts) => {
                let cell_name = parts.pop_front().expect("parent CellName");
                Some(match parts.len() {
                    0 => unreachable!(
                        "Path variant should only be constructed when length is > 1"
                    ),
                    1 => (cell_name, Self::CellName(parts.pop_front().expect("length is 1"))),
                    _ => (cell_name, Self::Path(parts)),
                })
            }
        }
    }

    pub fn into_string(self) -> String {
        match self {
            CellNamePath::Empty => "".into(),
            CellNamePath::CellName(cell_name) => cell_name.into_inner(),
            CellNamePath::Path(parts) => parts.into_iter().join(SEPARATOR),
        }
    }
}

impl ValidatedField<String> for CellNamePath {
    fn validate(
        input: Option<String>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input = validation::required(input, field_name, parent_name)?;

        if input.is_empty() {
            return Ok(Self::Empty);
        }

        let parts: Vec<_> = input.split(SEPARATOR).collect();

        if parts.len() == 1 {
            let cell_name = CellName::validate_for_creation(
                Some(input),
                field_name,
                parent_name,
            )?;

            Ok(Self::CellName(cell_name))
        } else {
            let parts = parts
                .into_iter()
                .flat_map(|cell_name| {
                    CellName::validate_for_creation(
                        Some(cell_name.into()),
                        field_name,
                        parent_name,
                    )
                })
                .collect();

            Ok(Self::Path(parts))
        }
    }
}
