use std::fmt;
use std::error;
use std::env::VarError;
use std::error::Error as StdError;
use std::num::ParseIntError;
use rustc_serialize::json::DecoderError;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    NoMatrix,
    NotLeader,
    BuildNotFound,
    FailedBuilds,
}

impl Error {
    pub fn from_str(message: &str) -> Error {
        Error::from_string(message.into())
    }

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
            Error::NoMatrix => "No matrix found. Call `build_matrix` first.",
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

impl From<DecoderError> for Error {
    fn from(err: DecoderError) -> Error {
        Error::Generic(err.description().into())
    }
}
