# travis-after-all

[![Build Status](https://travis-ci.org/badboy/travis-after-all-rs.svg?branch=master)](https://travis-ci.org/badboy/travis-after-all-rs)
[![crates.io](http://meritbadge.herokuapp.com/travis-after-all)](https://crates.io/crates/travis-after-all)

Check that all jobs in a build matrix run and succeeded and launch a single task afterwards.

Travis offers no way to launch a single task when all jobs in a build finish.
Relevant issue: <https://github.com/travis-ci/travis-ci/issues/929>

Sometimes such a hook is necessary, for example to publish a new version of your project only once
and only if all jobs succeed.

travis-after-all is a workaround for this and allows to wait for all jobs and then run a
command afterwards.
This is a port of the original Python script: <https://github.com/dmakhno/travis_after_all>


## [Documentation][]

[Documentation is available online.][Documentation]

[Documentation]: http://badboy.github.io/travis-after-all-rs

## CLI usage

You need to add the following lines to your `.travis.yml`.
This installs the tool and executes as an `after_success` hook:
(It will only work for Rust projects as it depends on Cargo, the Rust package manager)

```yaml
before_script:
  - |
     export PATH=$HOME/.cargo/bin:$PATH:$PATH &&
     cargo install --git https://github.com/badboy/travis-after-all-rs

after_success:
  - travis-after-all && echo "All fine, let's publish"
```

## Library usage

You can use it as a library as well to build your own hooks:

```rust
use travis_after_all::Build;
let build_run = Build::from_env().unwrap();
if build_run.is_leader() {
    let _ = build_run.wait_for_others().unwrap();
    println!("Everything done. We can work here now.");
}
```

## License

This project is licensed under the MIT license. See [LICENSE](LICENSE).

