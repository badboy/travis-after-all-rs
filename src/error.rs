use std::fmt;
use std::error;
use std::env::VarError;
use std::error::Error as StdError;
use std::num::ParseIntError;
use reqwest;

/// All possible error cases
#[derive(Debug)]
pub enum Error {
    /// A generic error, translated from another internal error
    Generic(String),
    /// `wait_for_others` was called on a non-leader environment
    NotLeader,
    /// The specified build was not found
    BuildNotFound,
    /// All non-leader jobs finished, but at least one failed
    FailedBuilds,
}

impl Error {
    /// Build an error from a string
    pub fn from_str(message: &str) -> Error {
        Error::from_string(message.into())
    }

    /// Build an error from a string
    pub fn from_string(message: String) -> Error {
        Error::Generic(message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(self, f)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Generic(ref s) => s,
            Error::NotLeader => "This build is not the leader",
            Error::FailedBuilds => "Some builds failed",
            Error::BuildNotFound => "This build does not exist",
        }
    }
}

impl From<VarError> for Error {
    fn from(err: VarError) -> Error {
        match err {
            VarError::NotPresent => Error::Generic("Environment variable not present".into()),
            VarError::NotUnicode(_) => Error::Generic("Environment variable not valid".into())
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(_err: ParseIntError) -> Error {
        Error::Generic("Can't parse what should be an integer".into())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Generic(err.description().into())
    }
}
