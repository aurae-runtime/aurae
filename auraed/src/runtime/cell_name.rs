use std::ops::Deref;
use validation::ValidationError;

#[derive(Debug)]
pub(crate) struct CellName(String);

impl validation::ValidatedField<String> for CellName {
    fn validate(
        input: Option<String>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input =
            validation::required_not_empty(input, field_name, parent_name)?;

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

        Ok(Self(input))
    }
}

impl Deref for CellName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
