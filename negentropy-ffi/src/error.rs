// Copyright (c) 2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;

pub type Result<T, E = NegentropyError> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum NegentropyError {
    Generic { err: String },
}

impl fmt::Display for NegentropyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic { err } => write!(f, "{err}"),
        }
    }
}

impl From<negentropy::Error> for NegentropyError {
    fn from(e: negentropy::Error) -> Self {
        Self::Generic { err: e.to_string() }
    }
}