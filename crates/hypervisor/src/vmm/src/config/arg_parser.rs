// Copyright 2021 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub(super) enum CfgArgParseError {
    /// Parsing failed, param and error.
    ParsingFailed(&'static str, String),
    UnknownArg(String),
}

impl fmt::Display for CfgArgParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParsingFailed(param, err) => {
                write!(f, "Param '{}', parsing failed: {}", param, err)
            }
            Self::UnknownArg(err) => write!(f, "Unknown arguments found: '{}'", err),
        }
    }
}

pub(super) struct CfgArgParser {
    args: HashMap<String, String>,
}

impl CfgArgParser {
    pub(super) fn new(input: &str) -> Self {
        let args = input
            .split(',')
            .filter(|tok| !tok.is_empty())
            .map(|tok| {
                let mut iter = tok.splitn(2, '=');
                let param_name = iter.next().unwrap();
                let value = iter.next().unwrap_or("").to_string();
                (param_name.to_lowercase(), value)
            })
            .collect();
        Self { args }
    }

    /// Retrieves the value of `param`, consuming it from `Self`.
    pub(super) fn value_of<T: FromStr>(
        &mut self,
        param_name: &'static str,
    ) -> Result<Option<T>, CfgArgParseError>
    where
        <T as FromStr>::Err: fmt::Display,
    {
        match self.args.remove(param_name) {
            Some(value) if !value.is_empty() => value
                .parse::<T>()
                .map_err(|err| CfgArgParseError::ParsingFailed(param_name, err.to_string()))
                .map(Some),
            _ => Ok(None),
        }
    }

    /// Checks if all params were consumed.
    pub(super) fn all_consumed(&self) -> Result<(), CfgArgParseError> {
        if self.args.is_empty() {
            Ok(())
        } else {
            Err(CfgArgParseError::UnknownArg(self.to_string()))
        }
    }
}

impl fmt::Display for CfgArgParser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.args
                .keys()
                .map(|val| val.as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU8;
    use std::path::PathBuf;

    #[test]
    fn test_cfg_arg_parse() -> Result<(), CfgArgParseError> {
        let input_params = "path=/path,string=HelloWorld,int=123,u8=1";
        let mut arg_parser = CfgArgParser::new(input_params);

        // No parameter was consumed yet
        assert!(arg_parser.all_consumed().is_err());

        assert_eq!(
            arg_parser.value_of::<PathBuf>("path")?.unwrap(),
            PathBuf::from("/path")
        );
        assert_eq!(
            arg_parser.value_of::<String>("string")?.unwrap(),
            "HelloWorld".to_string()
        );
        assert_eq!(
            arg_parser.value_of::<NonZeroU8>("u8")?.unwrap(),
            NonZeroU8::new(1).unwrap()
        );
        assert_eq!(arg_parser.value_of::<u64>("int")?.unwrap(), 123);

        // Params now is empty, use the Default instead.
        let default = 12;
        assert_eq!(arg_parser.value_of("int")?.unwrap_or(default), default);

        // Params is empty and no Default provided:
        assert!(arg_parser.value_of::<u64>("int")?.is_none());

        // All params were consumed:
        assert!(arg_parser.all_consumed().is_ok());

        let input_params = "path=";
        assert!(CfgArgParser::new(input_params)
            .value_of::<String>("path")?
            .is_none());

        Ok(())
    }
}
