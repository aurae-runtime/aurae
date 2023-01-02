//use fancy_regex::Regex;

use std::{
    fmt::{Debug, Display, Formatter},
    ops::Deref,
};

use iter_tools::Itertools;
use validation::{ValidatedField, ValidationError};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct CpuCpus(Vec<i32>);

impl CpuCpus {
    #[cfg(test)]
    pub fn new(cpu_cpus: Vec<i32>) -> Self {
        Self(cpu_cpus)
    }

    pub fn into_inner(self) -> Vec<i32> {
        self.0
    }

    pub fn as_string(&self) -> String {
        self.0.iter().map(|&id| id.to_string()).join(",")
    }
}

impl ValidatedField<Vec<i32>> for CpuCpus {
    fn validate(
        input: Option<Vec<i32>>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input = validation::required(input, field_name, parent_name)?;
        Ok(Self(input))
    }

    fn validate_for_creation(
        input: Option<Vec<i32>>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input = Self::validate(input, field_name, parent_name)?;

        // TODO: maybe lazy_static this
        // input should be a comma seperated list of numbers with optional -s
        // or the empty string.
        //let pattern: Regex =
        //    Regex::new(r"^(\d(-\d)?,?)*$").expect("regex construction");
        //validation::allow_regex(&input, &pattern, field_name, parent_name)?;

        Ok(input)
    }
}

impl Deref for CpuCpus {
    type Target = Vec<i32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for CpuCpus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_test_case::test_case;

    #[test_case(vec![], ""; "empty list")]
    #[test_case(vec![0], "0"; "just one cpu")]
    #[test_case(vec![1,2], "1,2"; "two")]
    #[test_case(vec![1,4,5], "1,4,5"; "a range")]
    #[test]
    fn test_validation_success(input: Vec<i32>, expectation: &str) {
        let input: CpuCpus = CpuCpus::new(input);
        assert_eq!(input.as_string(), expectation);
        assert!(CpuCpus::validate_for_creation(
            Some(input.into_inner()),
            "cpu_cpus",
            None
        )
        .is_ok());
    }
}
