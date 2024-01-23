# CaDiCaL SAT solver
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


## Information for developers

To update cadical version, simply download a new version from:
```
https://github.com/sirandreww/cadical.git
```
This is a fork of cadical that fixes a small issue with the C API of cadical.

Then:
1. Paste the downloaded to replace `cadical-b29a98e5f1fd93a3adb775a498a25b41e0cc70e7`
2. Update `CADICAL_PATH` in `build.rs`
3. Update include in `src/ccadical.cpp`

## Using different C++ compilers

To compile the project you need to have a C++ compiler in order to compile CaDiCal.
To set the compiler certain environment variables must be set before trying to compile.

The C++ standard library may be linked to the crate target. 
By default it's:
1. `libc++` for macOS, FreeBSD, and OpenBSD
2. `libc++_shared` for Android, nothing for MSVC
3. `libstdc++` for anything else. 
It can be changed by setting the `CXXSTDLIB` environment variable.

### Using c++

Run these commands in order.
```
cargo clean
unset CRATE_CC_NO_DEFAULTS
unset CXXFLAGS
unset CXXSTDLIB
export CXX=/usr/bin/g++
cargo test
```

### Using Clang

Run these commands in order.
```
sudo apt install libc++abi-dev libstdc++-11-dev
cargo clean
unset CRATE_CC_NO_DEFAULTS
export CXXFLAGS="-stdlib=libstdc++"
unset CXXSTDLIB
export CXX=clang++
cargo test
```

