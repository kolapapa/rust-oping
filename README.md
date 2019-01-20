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
separately. However, there are several dependencies required to build this:

* GNU autotools: `libtool`, `autoconf`, and `automake` (These can be found as
  packages in most GNU/Linux distributions, and in OS X's Homebrew system, for
  example.)
* A system C compiler (`gcc` or equivalent)

This crate was written by Chris Fallin &lt;cfallin@c1f.net&gt; and is released
under the MIT license.

Documentation is available [here](https://cfallin.github.io/rust-oping/oping/),
and the crate is available as `oping`
[on crates.io here](https://crates.io/crates/oping/).

*NOTE*: sending ping packets requires either running as `root` or setting a
capability on your binary, at least on Linux. This is a restriction enforced by
the system, not by this crate. To set the capability, run the following as
root:

    $ setcap cap_net_raw+ep $MY_BINARY    # allow binary to send ping packets
