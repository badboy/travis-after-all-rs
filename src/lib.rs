extern crate curl;
extern crate rustc_serialize;

use std::thread;
use std::time::Duration;
use std::env;
use std::str::FromStr;
use rustc_serialize::json;
use curl::http;

mod error;
pub use error::Error;

const USER_AGENT:         &'static str = concat!("travis-after-all/", env!("CARGO_PKG_VERSION"));
const TRAVIS_JOB_NUMBER : &'static str = "TRAVIS_JOB_NUMBER";
const TRAVIS_BUILD_ID:    &'static str = "TRAVIS_BUILD_ID";
const TRAVIS_API_URL:     &'static str = "https://api.travis-ci.org";
const POLLING_INTERVAL:   &'static str = "LEADER_POLLING_INTERVAL";

pub struct Build {
    travis_api_url: String,
    build_id: String,
    job_number: String,
    polling_interval: u64,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct MatrixElement {
    finished_at: Option<String>,
    result: Option<u32>,
    number: String,
    id: Option<u32>,
}

impl MatrixElement {
    pub fn id(&self) -> u32 {
        self.id.unwrap()
    }

    pub fn is_leader(&self) -> bool {
        is_leader(&self.number)
    }

    pub fn is_succeeded(&self) -> bool {
        if !self.is_finished() {
           return false;
        }

        match self.result {
            None => false,
            Some(0) => true,
            Some(_) => false,
        }
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

    pub fn others_succeeded(&self) -> bool {
        self.matrix.iter()
            .filter(|build| !build.is_leader())
            .all(|build| build.is_succeeded())
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
        })

    }

    pub fn is_leader(&self) -> bool {
        is_leader(&self.job_number)
    }

    pub fn build_matrix(&self) -> Result<Matrix, Error> {
        let url = format!("{}/builds/{}", self.travis_api_url, self.build_id);
        let res = http::handle()
            .get(url)
            .follow_redirects(true)
            .header("User-Agent", USER_AGENT)
            .exec()
            .unwrap();

        if res.get_code() == 404 {
            return Err(Error::BuildNotFound);
        }

        let body = String::from_utf8(res.move_body()).unwrap();
        Ok(try!(json::decode(&body)))
    }

    pub fn wait_for_others(&self) -> Result<(), Error> {
        if !self.is_leader() {
            return Err(Error::NotLeader)
        }

        let dur = Duration::new(self.polling_interval, 0);
        loop {
            let matrix = try!(self.build_matrix());

            if matrix.others_finished() {
                break;
            }
            thread::sleep(dur);
        }

        let matrix = try!(self.build_matrix());
        match matrix.others_succeeded() {
            true => Ok(()),
            false => Err(Error::FailedBuilds)
        }
    }
}

