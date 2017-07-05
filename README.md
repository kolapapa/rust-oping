liboping bindings for Rust: simple ICMP pings
=============================================

This crate is a simple Rust binding for [liboping](http://noping.cc/), a
library that implements basic ICMP ping functionality. These bindings allow a
Rust program to send ping packets (possibly to multiple hosts in parallel) and
enumerate the responses.

This crate also includes a very simple program `rustping` that uses the
bindings to implement a barebones command-line ping utility.

This crate includes `liboping` in its source tree (as a submodule) and builds
it into the Rust library, so there is no need to build and install it
separately.

This crate was written by Chris Fallin &lt;cfallin@c1f.net&gt; and is released
under the MIT license.

Documentation is available [here](https://cfallin.github.io/rust-oping/oping/),
and the crate is available as `oping`
[on crates.io here](https://crates.io/crates/oping/).
