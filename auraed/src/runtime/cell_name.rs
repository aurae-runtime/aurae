use std::fmt::{Display, Formatter};
use std::ops::Deref;
use validation::{ValidatedField, ValidationError};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct CellName(String);

impl CellName {
    #[cfg(test)]
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn into_inner(self) -> String {
        self.0
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

        Ok(Self(input))
    }

    fn validate_for_creation(
        input: Option<String>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input = Self::validate(input, field_name, parent_name)?;

        // TODO: what makes a valid cgroup name
        // any valid path?
        // do we want a restriction on length?
        // anything else?
        // Highly restrictive for now:
        validation::allow_regex(
            &input,
            &validation::DOMAIN_NAME_LABEL_REGEX,
            field_name,
            parent_name,
        )?;

        Ok(input)
    }
}

impl Deref for CellName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for CellName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
