extern crate curl;
extern crate rustc_serialize;

use std::thread;
use std::time::Duration;
use std::env::{self, VarError};
use std::str::FromStr;
use std::num::ParseIntError;
use rustc_serialize::json;
use curl::http;

const TRAVIS_JOB_NUMBER : &'static str = "TRAVIS_JOB_NUMBER";
const TRAVIS_BUILD_ID:    &'static str = "TRAVIS_BUILD_ID";
const TRAVIS_API_URL:     &'static str = "https://api.travis-ci.org";
const TRAVIS_TOKEN:       &'static str = "TRAVIS_TOKEN";
const POLLING_INTERVAL:   &'static str = "LEADER_POLLING_INTERVAL";
const GITHUB_TOKEN1:      &'static str = "GITHUB_TOKEN";
const GITHUB_TOKEN2:      &'static str = "GH_TOKEN";

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
    gh_token: String,
    travis_token: Option<String>,
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

#[derive(RustcDecodable, RustcEncodable)]
struct TokenFetcher {
    github_token: String
}

pub fn is_leader(job: &str) -> bool {
    job.ends_with('1')
}

impl Build {
    pub fn from_env() -> Result<Build, Error> {
        let build_id = try!(env::var(TRAVIS_BUILD_ID));
        let job_number = try!(env::var(TRAVIS_JOB_NUMBER));
        let gh_token = match env::var(GITHUB_TOKEN1) {
            Ok(token) => token,
            Err(_) => try!(env::var(GITHUB_TOKEN2)),
        };

        let polling_interval = match env::var(POLLING_INTERVAL) {
            Err(_) => 5,
            Ok(val) => try!(FromStr::from_str(&val))
        };

        let travis_token = env::var(TRAVIS_TOKEN).ok();

        Ok(Build {
            travis_api_url: TRAVIS_API_URL.into(),
            build_id: build_id,
            job_number: job_number,
            gh_token: gh_token,
            travis_token: travis_token,
            polling_interval: polling_interval,
            matrix: None,
        })

    }

    pub fn get_token(&mut self) -> String { // }Result<&str, ()> {
        let data = TokenFetcher { github_token: self.gh_token.clone() };
        let data = json::encode(&data).unwrap();
        let url = format!("{}/auth/github", self.travis_api_url);
        let res = http::handle()
            .post(url, &data)
            .header("Content-Type", "application/json")
            .exec()
            .unwrap();

        String::from_utf8(res.move_body()).unwrap()
    }

    pub fn is_leader(&self) -> bool {
        is_leader(&self.job_number)
    }

    pub fn build_matrix(&mut self) -> &Matrix {
        if self.matrix.is_some() {
            return self.matrix.as_ref().unwrap();
        }
        let url = format!("{}/builds/{}", self.travis_api_url, self.build_id);
        let token = self.travis_token.clone().unwrap_or("foo".into());
        let res = http::handle()
            .get(url)
            .follow_redirects(true)
            .header("Authorization", &format!("token {}", token))
            .header("Content-Type", "application/json")
            .exec()
            .unwrap();

        let body = String::from_utf8(res.move_body()).unwrap();
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

