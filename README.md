CaDiCaL SAT solver
==================
[![Build Status](https://travis-ci.com/mmaroti/cadical-rs.svg?branch=master)](https://travis-ci.com/mmaroti/cadical-rs)
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
sat.add_clause([1, 2].iter().copied());
sat.add_clause([-1, 2].iter().copied());
assert_eq!(sat.solve(), Some(true));
assert_eq!(sat.value(2), Some(true));
```