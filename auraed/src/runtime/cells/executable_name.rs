use std::fmt::{Display, Formatter};
use std::ops::Deref;
use validation::{ValidatedField, ValidationError};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct ExecutableName(String);

impl ExecutableName {
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl ValidatedField<String> for ExecutableName {
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

        // TODO: what makes a valid executable name
        // Wasn't there something about 16 bytes (including terminating 0 byte) and anything more would be silently truncated.
        // We don't want to silently truncate IMO, if that is the case.
        //
        // validation::maximum_length(
        //     input.as_bytes(),
        //     15,
        //     "bytes",
        //     field_name,
        //     parent_name,
        // )?;

        Ok(input)
    }
}

impl Deref for ExecutableName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ExecutableName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}