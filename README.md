CaDiCaL SAT solver
==================
[![Build Status](https://app.travis-ci.com/mmaroti/cadical-rs.svg?branch=master)](https://app.travis-ci.com/github/mmaroti/cadical-rs)
[![Crate](https://img.shields.io/crates/v/cadical)](https://crates.io/crates/cadical)
[![Documentation](https://docs.rs/cadical/badge.svg)](https://docs.rs/cadical)
[![GitHub](https://img.shields.io/github/license/mmaroti/cadical-rs)](LICENSE)

This is a stand alone crate that contains both the C++ source code of the
CaDiCaL incremental SAT solver together with its Rust binding. The C++
files are compiled and statically linked during the build process. This
crate works on Linux, Apple OSX, Windows, Android, iOS, Raspberry Pi,
NetBSD and FreeBSD.

CaDiCaL won first place in the SAT track of the SAT Race 2019 and second
overall place. It was written by Armin Biere, and it is available under the
MIT license.

The literals are unwrapped positive and negative integers, exactly as in the
DIMACS format. The common IPASIR operations are presented in a safe Rust
interface.

```
let mut sat: cadical::Solver = Default::default();
sat.add_clause([1, 2]);
sat.add_clause([-1, 2]);
assert_eq!(sat.solve(), Some(true));
assert_eq!(sat.value(2), Some(true));
```

The C++ library is build with assertions disabled and with optimization level
3 by default. C++ assertions are enabled only when cargo is building a debug 
version and the `cpp-debug` feature of the library is enabled.
