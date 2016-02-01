extern crate curl;
extern crate rustc_serialize;

use std::thread;
use std::time::Duration;
use std::env::{self, VarError};
use std::str::FromStr;
use std::num::ParseIntError;
use rustc_serialize::json;
use curl::http;

const USER_AGENT:         &'static str = concat!("travis-after-all/", env!("CARGO_PKG_VERSION"));
const TRAVIS_JOB_NUMBER : &'static str = "TRAVIS_JOB_NUMBER";
const TRAVIS_BUILD_ID:    &'static str = "TRAVIS_BUILD_ID";
const TRAVIS_API_URL:     &'static str = "https://api.travis-ci.org";
const POLLING_INTERVAL:   &'static str = "LEADER_POLLING_INTERVAL";

#[derive(Debug)]
pub enum Error {
    Generic(String),
    NoMatrix,
    NotLeader,
}

impl Error {
    pub fn from_str(message: &str) -> Error {
        Error::Generic(message.into())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        std::fmt::Debug::fmt(self, f)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Generic(ref s) => s,
            Error::NoMatrix => "No matrix found. Call `build_matrix` first.",
            Error::NotLeader => "This build is not the leader"
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

pub struct Build {
    travis_api_url: String,
    build_id: String,
    job_number: String,
    polling_interval: u64,
    matrix: Option<Matrix>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct MatrixElement {
    finished_at: Option<String>,
    result: u32,
    number: String,
    id: u32,
}

impl MatrixElement {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn is_leader(&self) -> bool {
        is_leader(&self.number)
    }

    pub fn is_succeeded(&self) -> bool {
        self.is_finished() && self.result == 0
    }

    pub fn is_finished(&self) -> bool {
        self.finished_at.is_some()
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Matrix {
    id: u32,
    matrix: Vec<MatrixElement>,
}

impl Matrix {
    pub fn builds(&self) -> &[MatrixElement] {
        &self.matrix
    }

    pub fn others_finished(&self) -> bool {
        self.matrix.iter()
            .filter(|build| !build.is_leader())
            .all(|build| build.is_finished())
    }
}

pub fn is_leader(job: &str) -> bool {
    job.ends_with('1')
}

impl Build {
    pub fn from_env() -> Result<Build, Error> {
        let build_id = try!(env::var(TRAVIS_BUILD_ID));
        let job_number = try!(env::var(TRAVIS_JOB_NUMBER));

        let polling_interval = match env::var(POLLING_INTERVAL) {
            Err(_) => 5,
            Ok(val) => try!(FromStr::from_str(&val))
        };

        Ok(Build {
            travis_api_url: TRAVIS_API_URL.into(),
            build_id: build_id,
            job_number: job_number,
            polling_interval: polling_interval,
            matrix: None,
        })

    }

    pub fn is_leader(&self) -> bool {
        is_leader(&self.job_number)
    }

    pub fn build_matrix(&mut self) -> &Matrix {
        if self.matrix.is_some() {
            return self.matrix.as_ref().unwrap();
        }
        let url = format!("{}/builds/{}", self.travis_api_url, self.build_id);
        let res = http::handle()
            .get(url)
            .follow_redirects(true)
            .header("User-Agent", USER_AGENT)
            .exec()
            .unwrap();

        let body = String::from_utf8(res.move_body()).unwrap();
        println!("=== BODY ===");
        println!("{}", body);
        println!("");
        println!("=== /BODY ===");
        self.matrix = Some(json::decode(&body).unwrap());
        self.matrix.as_ref().unwrap()
    }

    pub fn wait_for_others(&self) -> Result<(), Error> {
        if self.matrix.is_none() {
            return Err(Error::NoMatrix);
        }

        if self.is_leader() {
            return Err(Error::NotLeader)
        }

        let matrix = self.matrix.as_ref().unwrap();
        let dur = Duration::new(self.polling_interval, 0);
        while !matrix.others_finished() {
            thread::sleep(dur);
        }

        Ok(())
    }
}

