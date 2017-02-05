//! Check that all jobs in a build matrix run and succeeded and launch a single task afterwards.
//!
//! Travis offers no way to launch a single task when all jobs in a build finish.
//! Relevant issue: <https://github.com/travis-ci/travis-ci/issues/929>
//!
//! Sometimes such a hook is necessary, for example to publish a new version of your project only once
//! and only if all jobs succeed.
//!
//! travis-after-all is a workaround for this and allows to wait for all jobs and then run a
//! command afterwards.
//! This is a port of the original Python script: <https://github.com/dmakhno/travis_after_all>
//!
//! ## CLI usage
//!
//! You need to add the following lines to your `.travis.yml`.
//! This installs the tool and executes as an `after_success` hook:
//! (It will only work for Rust projects as it depends on Cargo, the Rust package manager)
//!
//! ```yaml
//! before_script:
//!   - |
//!      export PATH=$HOME/.cargo/bin:$PATH:$PATH &&
//!      cargo install --git https://github.com/badboy/travis-after-all-rs
//!
//! after_success:
//!   - travis-after-all && echo "All fine, let's publish"
//! ```
//!
//! ## Library usage
//!
//! You can use it as a library as well to build your own hooks:
//!
//! ```rust,no_run
//! use travis_after_all::Build;
//! let build_run = Build::from_env().unwrap();
//! if build_run.is_leader() {
//!     let _ = build_run.wait_for_others().unwrap();
//!     println!("Everything done. We can work here now.");
//! }
//! ```
#![deny(missing_docs)]

extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::thread;
use std::time::Duration;
use std::env;
use std::str::FromStr;
use reqwest::{RedirectPolicy, StatusCode};
use reqwest::header::UserAgent;

mod error;
mod matrix;
pub use error::Error;
pub use matrix::{Matrix, MatrixElement};

const USER_AGENT:         &'static str = concat!("travis-after-all/", env!("CARGO_PKG_VERSION"));
const TRAVIS_JOB_NUMBER : &'static str = "TRAVIS_JOB_NUMBER";
const TRAVIS_BUILD_ID:    &'static str = "TRAVIS_BUILD_ID";
const TRAVIS_API_URL:     &'static str = "https://api.travis-ci.org";
const POLLING_INTERVAL:   &'static str = "LEADER_POLLING_INTERVAL";


fn env_var(varname: &str) -> Result<String, Error> {
    env::var(varname)
        .map_err(|_| Error::from_string(format!("Missing environment variable: {}", varname)))
}

fn is_leader(job: &str) -> bool {
    job.ends_with('1')
}

/// The information of a full build
pub struct Build {
    travis_api_url: String,
    build_id: String,
    job_number: String,
    polling_interval: u64,
}

impl Build {
    /// Fetch the relevant information from the environment
    ///
    /// This reads the environment variables `TRAVIS_BUILD_ID` and `TRAVIS_JOB_NUMBER`,
    /// which are automatically set by Travis.
    ///
    /// It also reads the variable `POLLING_INTERVAL` with a default of 5.
    /// Set it to a higher value for a longer timeout.
    pub fn from_env() -> Result<Build, Error> {
        let build_id = try!(env_var(TRAVIS_BUILD_ID));
        let job_number = try!(env_var(TRAVIS_JOB_NUMBER));

        let polling_interval = match env_var(POLLING_INTERVAL) {
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

    /// Whether or not the current environment is the build leader
    pub fn is_leader(&self) -> bool {
        is_leader(&self.job_number)
    }

    /// Fetch the build matrix for the current build
    pub fn build_matrix(&self) -> Result<Matrix, Error> {
        let url = format!("{}/builds/{}", self.travis_api_url, self.build_id);
        let mut client = reqwest::Client::new().unwrap();
        client.redirect(RedirectPolicy::limited(5));
        let mut res = client.get(&url)
            .header(UserAgent(USER_AGENT.to_string()))
            .send()
            .unwrap();

        if *res.status() == StatusCode::NotFound {
            return Err(Error::BuildNotFound);
        }

        res.json().map_err(|e| From::from(e))
    }

    /// Wait for all non-leader jobs to finish
    ///
    /// Returns an `Error::BuildNotFound` if this build is not known to Travis.
    /// Returns an `Error::FailedBuilds` if at least one non-leader build failed.
    ///
    /// This loops until it fails or succeeds, there is no way to exit the loop.
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
