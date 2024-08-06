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
use iter_tools::Itertools;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use validation::{ValidatedField, ValidationError};

pub const SEPARATOR: char = '/';

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CellName(PathBuf);

impl CellName {
    pub fn leaf(&self) -> Cow<str> {
        self.0
            .components()
            .last()
            .expect("non empty path")
            .as_os_str()
            .to_string_lossy()
    }

    pub fn to_root(&self) -> CellName {
        let root = self.0.components().find_or_first(|_| true).expect("root");
        Self(PathBuf::from(root.as_os_str()))
    }

    pub fn to_child(&self, descendant: &CellName) -> Option<CellName> {
        if descendant.is_child(Some(self)) {
            return Some(descendant.clone());
        }

        let mut path = &*descendant.0;
        while let Some(parent_path) = path.parent() {
            let descendant = CellName(parent_path.to_path_buf());
            if descendant.is_child(Some(self)) {
                return Some(descendant);
            } else {
                path = parent_path
            }
        }

        None
    }

    pub fn is_child(&self, parent: Option<&CellName>) -> bool {
        let self_parent = self.0.parent().filter(|x| !x.as_os_str().is_empty());

        match (self_parent, parent) {
            (None, None) => true,
            (None, _) | (_, None) => false,
            (Some(self_parent), Some(parent)) => {
                self_parent == parent.0.as_path()
            }
        }
    }

    pub fn into_inner(self) -> PathBuf {
        self.0
    }

    pub fn as_inner(&self) -> &Path {
        self.0.as_path()
    }

    #[cfg(test)]
    pub fn random_for_tests() -> Self {
        Self(PathBuf::from(format!("ae-test-{}", uuid::Uuid::new_v4())))
    }

    #[cfg(test)]
    pub fn random_child_for_tests(parent: &CellName) -> Self {
        Self(parent.0.join(format!("ae-test-{}", uuid::Uuid::new_v4())))
    }
}

impl ValidatedField<String> for CellName {
    fn validate(
        input: Option<String>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input =
            validation::required_not_empty(input, field_name, parent_name)?;

        // Opting to be forgiving of paths that start or end with SEPARATOR
        let input = input.trim_matches(SEPARATOR);

        let input = input
            .split(SEPARATOR)
            .flat_map(|component| {
                // NOTE: We must always reserve '/' (separator) and '_' (name of leaf cgroup)
                validation::allow_regex(
                    component,
                    &validation::DOMAIN_NAME_LABEL_REGEX,
                    field_name,
                    parent_name,
                )?;

                Ok::<_, ValidationError>(component)
            })
            .collect();

        Ok(Self(input))
    }
}

impl Display for CellName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.display().fmt(f)
    }
}

#[cfg(test)]
impl From<&str> for CellName {
    fn from(x: &str) -> Self {
        CellName(x.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_leaf_top_level() {
        const CELL_NAME: &str = "top-level-cell";

        let cell_name =
            CellName::validate(Some(CELL_NAME.into()), "test", None)
                .expect("failed to create valid cell name");

        assert_eq!(cell_name.leaf(), CELL_NAME);
    }

    #[test]
    fn test_leaf_nested_level() {
        let cell_name = CellName::validate(
            Some("grandparent-cell/parent-cell/child-cell".into()),
            "test",
            None,
        )
        .expect("failed to create valid cell name");

        assert_eq!(cell_name.leaf(), "child-cell");
    }

    #[test]
    fn test_to_root() {
        let cell_name = CellName::validate(
            Some("grandparent-cell/parent-cell/child-cell".into()),
            "test",
            None,
        )
        .expect("failed to create valid cell name");

        assert_eq!(
            cell_name.to_root(),
            CellName(PathBuf::from_str("grandparent-cell").unwrap())
        );
    }

    #[test]
    fn test_is_child() {
        let grandparent_cell_name =
            CellName(PathBuf::from_str("grandparent-cell").unwrap());

        assert!(grandparent_cell_name.is_child(None));

        let parent_cell_name = CellName(
            PathBuf::from_str("grandparent-cell/parent-cell").unwrap(),
        );

        assert!(parent_cell_name.is_child(Some(&grandparent_cell_name)));
        assert!(!parent_cell_name.is_child(None));

        let child_cell_name = CellName(
            PathBuf::from_str("grandparent-cell/parent-cell/child-cell")
                .unwrap(),
        );

        assert!(child_cell_name.is_child(Some(&parent_cell_name)));
        assert!(!child_cell_name.is_child(Some(&grandparent_cell_name)));
        assert!(!child_cell_name.is_child(None));
    }

    #[test]
    fn test_to_child() {
        let grandparent_cell_name =
            CellName(PathBuf::from_str("grandparent-cell").unwrap());

        let parent_cell_name = CellName(
            PathBuf::from_str("grandparent-cell/parent-cell").unwrap(),
        );

        let child_cell_name = CellName(
            PathBuf::from_str("grandparent-cell/parent-cell/child-cell")
                .unwrap(),
        );

        let child_of_grandparent =
            grandparent_cell_name.to_child(&parent_cell_name).expect(
                "failed to create child_of_grandparent using `parent_cell_name`",
            );

        assert_eq!(child_of_grandparent, parent_cell_name);

        let child_of_grandparent =
            grandparent_cell_name.to_child(&child_cell_name).expect(
                "failed to create child_of_grandparent using `child_cell_name`",
            );

        assert_eq!(child_of_grandparent, parent_cell_name);
    }
}