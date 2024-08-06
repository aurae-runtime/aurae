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
#![warn(future_incompatible, nonstandard_style, unused)]
#![warn(clippy::unwrap_used)]

#[cfg(feature = "regex")]
pub use self::allow_regex::allow_regex;
pub use self::maximum_length::maximum_length;
pub use self::maximum_value::maximum_value;
pub use self::minimum_length::minimum_length;
pub use self::minimum_value::minimum_value;
pub use self::required::required;
pub use self::required_not_empty::required_not_empty;
#[cfg(feature = "secrecy")]
pub use self::required_not_empty::required_not_empty_secret_string;
pub use self::valid_enum::valid_enum;
#[cfg(feature = "json")]
pub use self::valid_json::valid_json;
#[cfg(feature = "url")]
pub use self::valid_url::valid_url;
#[cfg(feature = "regex")]
use fancy_regex::Regex;
#[cfg(feature = "regex")]
use lazy_static::lazy_static;

#[cfg(feature = "regex")]
mod allow_regex;
mod maximum_length;
mod maximum_value;
mod minimum_length;
mod minimum_value;
mod required;
mod required_not_empty;
mod valid_enum;
#[cfg(feature = "json")]
mod valid_json;
#[cfg(feature = "url")]
mod valid_url;

pub const UNIT_BYTES: &str = "bytes";
pub const UNIT_CHARACTER: &str = "character";
pub const UNIT_CHARACTERS: &str = "characters";
pub const UNIT_ITEM: &str = "item";
pub const UNIT_ITEMS: &str = "items";

#[cfg(feature = "regex")]
lazy_static! {
    pub static ref DOMAIN_NAME_LABEL_REGEX: Regex =
        Regex::new(r"^(?=.{1,63}$)(?![-])[a-zA-Z0-9-]+(?<![-])$")
            .expect("failed to parse 'DOMAIN_NAME_LABEL_REGEX'");
    pub static ref UNRESERVED_URL_PATH_SEGMENT_REGEX: Regex =
        Regex::new(r"^(?=.{1,1745}$)[a-zA-Z0-9_.~-]+$")
            .expect("failed to parse 'UNRESERVED_URL_PATH_SEGMENT_REGEX'");
}

pub trait ValidatingType<T> {
    fn validate(self, parent_name: Option<&str>) -> Result<T, ValidationError>;
}

pub trait ValidatedType<T> {
    fn validate(
        input: T,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError>
    where
        Self: Sized;
}

pub trait ValidatedField<T>
where
    Self: Sized,
{
    fn validate(
        input: Option<T>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError>;

    /// Default implementation returns immediately when input is `None`.
    /// Otherwise, calls `Self::validate`.
    fn validate_optional(
        input: Option<T>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Option<Self>, ValidationError> {
        if input.is_none() {
            return Ok(None);
        }

        Ok(Some(Self::validate(input, field_name, parent_name)?))
    }

    /// Default implementation calls `Self::validate`.
    ///
    /// The purpose of this validate function is to provide a place to run stricter validation,
    /// which is often wanted when creating things.
    fn validate_for_creation(
        input: Option<T>,
        field_name: &str,
        parent_name: Option<&str>,
    ) -> Result<Self, ValidationError> {
        Self::validate(input, field_name, parent_name)
    }
}

pub fn field_name(field_name: &str, parent_name: Option<&str>) -> String {
    match parent_name {
        None => field_name.to_string(),
        Some(parent_name) => format!("{parent_name}.{field_name}"),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ValidationError {
    #[error("Field = {field}; Required")]
    Required { field: String },
    #[error("Field = {field}; Minimum = {minimum} {units}")]
    Minimum { field: String, minimum: String, units: String },
    #[error("Field = {field}; Maximum = {maximum} {units}")]
    Maximum { field: String, maximum: String, units: String },
    #[cfg(feature = "regex")]
    #[error("Field = {field};  Regex = {pattern}")]
    AllowRegexViolation { field: String, pattern: String },
    #[error("Field = {field}; Invalid")]
    Invalid { field: String },
}

impl ValidationError {
    pub fn get_field(&self) -> &str {
        match self {
            Self::Required { field }
            | Self::Minimum { field, .. }
            | Self::Maximum { field, .. }
            | Self::Invalid { field, .. } => field,
            #[cfg(feature = "regex")]
            Self::AllowRegexViolation { field, .. } => field,
        }
    }
}

#[cfg(feature = "tonic")]
impl From<ValidationError> for tonic::Status {
    fn from(e: ValidationError) -> Self {
        Self::failed_precondition(e.to_string())
    }
}