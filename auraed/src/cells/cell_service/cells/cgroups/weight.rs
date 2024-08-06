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
use std::{
    fmt::{Display, Formatter},
    ops::Deref,
};

use validation::{ValidatedField, ValidationError};

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Weight(u64);

impl Weight {
    #[cfg(test)]
    pub fn new(weight: u64) -> Self {
        Self(weight)
    }

    pub fn into_inner(self) -> u64 {
        self.0
    }
}

impl ValidatedField<u64> for Weight {
    fn validate(
        input: Option<u64>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input: u64 = validation::required(input, field_name, parent_name)?;

        validation::minimum_value(input, 1, "unit", field_name, parent_name)?;

        validation::maximum_value(
            input,
            10000,
            "units",
            field_name,
            parent_name,
        )?;

        Ok(Self(input))
    }
}

impl Deref for Weight {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Weight {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_success() {
        assert!(
            Weight::validate_for_creation(Some(100), "weight", None).is_ok()
        );
    }

    #[test]
    fn test_validation_failure() {
        assert!(matches!(
            Weight::validate_for_creation(Some(0), "weight", None),
            Err(ValidationError::Minimum { .. })
        ));

        assert!(matches!(
            Weight::validate_for_creation(Some(10001), "weight", None),
            Err(ValidationError::Maximum { .. })
        ));
    }
}