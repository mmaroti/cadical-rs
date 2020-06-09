CaDiCaL SAT solver
==================

This is a stand alone crate that contains both the C++ source code of the
CaDiCaL incremental SAT solver together with its Rust binding. The C++
files are compiled and statically linked during the build process. This
crate works on Linux, Apple and Windows.

CaDiCaL won first place in the SAT track of the SAT Race 2019 and second
overall place. It was written by Armin Biere, and it is available under the
MIT license.

The literals are unwrapped positive and negative integers, exactly as in the
DIMACS format. The common IPASIR operations are presented in a safe Rust
interface.

```
let mut sat = cadical::Solver::new();
sat.add_clause([1, 2].iter().copied());
assert_eq!(sat.solve_with([-1].iter().copied()), Some(true));
assert_eq!(sat.value(1), Some(false));
assert_eq!(sat.value(2), Some(true));
```
