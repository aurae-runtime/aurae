use std::{
    fmt::{Display, Formatter},
    ops::Deref,
};

use validation::{ValidatedField, ValidationError};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct CpuWeight(u64);

impl CpuWeight {
    #[cfg(test)]
    pub fn new(cpu_weight: u64) -> Self {
        Self(cpu_weight)
    }

    pub fn into_inner(self) -> u64 {
        self.0
    }
}

impl ValidatedField<u64> for CpuWeight {
    fn validate(
        input: Option<u64>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input: u64 = validation::required(input, field_name, parent_name)?;
        Ok(Self(input))
    }

    fn validate_for_creation(
        input: Option<u64>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input: CpuWeight = Self::validate(input, field_name, parent_name)?;

        // see cpu weight in https://docs.kernel.org/admin-guide/cgroup-v2.html
        validation::maximum_value(
            *input,
            10000,
            "units",
            field_name,
            parent_name,
        )?;
        Ok(input)
    }
}

impl Deref for CpuWeight {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for CpuWeight {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let input: CpuWeight = CpuWeight::new(100);
        assert!(CpuWeight::validate_for_creation(
            Some(input.into_inner()),
            "cpu_weight",
            None
        )
        .is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let input: CpuWeight = CpuWeight::new(1000000);
        assert!(CpuWeight::validate_for_creation(
            Some(input.into_inner()),
            "cpu_weight",
            None
        )
        .is_err());
    }
}
