use std::{
    fmt::{Display, Formatter},
    ops::Deref,
};

use validation::{ValidatedField, ValidationError};

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct CpuQuota(i64);

impl CpuQuota {
    #[cfg(test)]
    pub fn new(cpu_quota: i64) -> Self {
        Self(cpu_quota)
    }

    pub fn into_inner(self) -> i64 {
        self.0
    }
}

impl ValidatedField<i64> for CpuQuota {
    fn validate(
        input: Option<i64>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input = validation::required(input, field_name, parent_name)?;
        Ok(Self(input))
    }

    fn validate_for_creation(
        input: Option<i64>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input = Self::validate(input, field_name, parent_name)?;

        validation::maximum_value(
            *input,
            1000000,
            "microseconds",
            field_name,
            parent_name,
        )?;
        Ok(input)
    }
}

impl Deref for CpuQuota {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for CpuQuota {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        let input: CpuQuota = CpuQuota::new(100000);
        assert!(CpuQuota::validate_for_creation(
            Some(input.into_inner()),
            "cpu_quota",
            None
        )
        .is_ok());
    }

    #[test]
    fn test_validation_failure() {
        let input: CpuQuota = CpuQuota::new(2000000);
        assert!(CpuQuota::validate_for_creation(
            Some(input.into_inner()),
            "cpu_quota",
            None
        )
        .is_err());
    }
}
