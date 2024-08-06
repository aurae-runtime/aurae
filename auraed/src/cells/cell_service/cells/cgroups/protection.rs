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

use std::fmt::{Display, Formatter};
use std::ops::Deref;
use validation::{ValidatedField, ValidationError};

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Protection(i64);

impl Protection {
    #[cfg(test)]
    pub fn new(allocation: i64) -> Self {
        Self(allocation)
    }

    pub fn into_inner(self) -> i64 {
        self.0
    }
}

impl ValidatedField<i64> for Protection {
    fn validate(
        input: Option<i64>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        let input = validation::required(input, field_name, parent_name)?;

        validation::minimum_value(input, 0, "units", field_name, parent_name)?;

        Ok(Self(input))
    }
}

impl Deref for Protection {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Protection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}