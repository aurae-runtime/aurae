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

    pub fn to_child(&self, descendant: &CellName) -> CellName {
        assert!(descendant.0.starts_with(self.0.as_path()));
        let path = self.0.join(descendant.to_root().0);
        Self(path)
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
