extern crate travis_after_all;

use std::process;
use std::error::Error as StdError;
use travis_after_all::{Build, Error};


fn human<T>(val: Result<T, Error>) -> T {
    match val {
        Ok(val) => val,
        Err(e) => {
            println!("travis_after_all failed.");
            println!("");
            println!("Error: {}", e.description());
            process::exit(1);
        }
    }

}

fn main() {
    let config = human(Build::from_env());

    if config.is_leader() {
        println!("I'm the leader.");
        println!("Waiting for others to finish");
        match config.wait_for_others() {
            Ok(()) => println!("Build finished. Now it's my time"),
            Err(Error::NotLeader) => {
                println!("I'm not the leader. Bailing out.");
                process::exit(3);
            },
            Err(Error::FailedBuilds) => {
                println!("Some builds failed. Stopping here.");
                process::exit(2);
            },
            Err(e) => {
                println!("travis_after_all failed.");
                println!("");
                println!("Error: {}", e.description());
                process::exit(1);
            }
        }

        {
            let matrix = config.build_matrix();

            if let Ok(matrix) = matrix {
                for build in matrix.builds() {
                    println!("== Build {}", build.id());
                    println!("Leader: {}", build.is_leader());
                    println!("Finished: {}", build.is_finished());
                    println!("Succeeded: {}", build.is_succeeded());
                    println!("==");
                    println!("");

                }
                println!("Matrix all except master finished: {}", matrix.others_finished());
                println!("Matrix all except master succeeded: {}", matrix.others_succeeded());
            }
        }
    }
}
