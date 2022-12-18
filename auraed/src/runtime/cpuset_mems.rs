use fancy_regex::Regex;

use std::{
    fmt::{Display, Formatter},
    ops::Deref,
};

use validation::{ValidatedField, ValidationError};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct CpusetMems(String);

impl CpusetMems {
    #[cfg(test)]
    pub fn new(cpu_cpus: String) -> Self {
        Self(cpu_cpus)
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl ValidatedField<String> for CpusetMems {
    fn validate(
        input: Option<String>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input = validation::required(input, field_name, parent_name)?;
        Ok(Self(input))
    }

    fn validate_for_creation(
        input: Option<String>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input = Self::validate(input, field_name, parent_name)?;

        // TODO: maybe lazy_static this
        // input should be a comma seperated list of numbers with optional -s
        // or the empty string.
        let pattern: Regex =
            Regex::new(r"^(\d(-\d)?,?)*$").expect("regex construction");
        validation::allow_regex(&input, &pattern, field_name, parent_name)?;

        Ok(input)
    }
}

impl Deref for CpusetMems {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for CpusetMems {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_test_case::test_case;

    #[test_case(""; "empty string")]
    #[test_case("0"; "just one cpu")]
    #[test_case("1,2"; "comma seperation")]
    #[test_case("1-3"; "a range")]
    #[test_case("1,2-5,6"; "combo")]
    #[test]
    fn test_validation_success(input: &str) {
        let input: CpusetMems = CpusetMems::new(input.into());
        assert!(CpusetMems::validate_for_creation(
            Some(input.into_inner()),
            "cpu_cpus",
            None
        )
        .is_ok());
    }

    #[test_case("foo"; "text")]
    #[test_case("1:2"; "colon seperation")]
    #[test_case("1..3"; "not a range")]
    #[test_case("1,foo;5"; "bad combo")]
    #[test]
    fn test_validation_failure(input: &str) {
        let input: CpusetMems = CpusetMems::new(input.into());
        assert!(CpusetMems::validate_for_creation(
            Some(input.into_inner()),
            "cpu_cpus",
            None
        )
        .is_err());
    }
}
