# v2.0.0 (2017-02-05)

Switch from `curl` & `rustc_serialize` to `reqwest` & `serde`

This makes it far more future proof and additionally does not require
openssl on Mac or Windows!

# v1.0.0 (2016-02-02)

The missing `after_all_success` hook for Travis

Did you ever wanted to run a task after all jobs of your build finished and succeeded?
Then this is for you.

Travis doesn't offer a `after_all_success` hook, so it is necessary to work around that.
This tool allows to wait for all builds and run a single task on the build leader,
that is on the node of the first job in the build matrix

See the README on <https://github.com/badboy/travis-after-all-rs> for more information.

Documentation is online at <http://badboy.github.io/travis-after-all-rs>

This is a port of the original Python script: <https://github.com/dmakhno/travis_after_all>
