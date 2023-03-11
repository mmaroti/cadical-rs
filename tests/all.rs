#[cfg(test)]
mod tests {
    use cadical::{Error, Solver, Timeout};
    use std::{iter, path::Path, thread, time::Instant};

    #[test]
    fn solver() {
        let mut sat: Solver = Solver::new();
        assert!(sat.signature().starts_with("cadical-"));
        assert_eq!(sat.status(), None);
        sat.add_clause([1, 2]);
        assert_eq!(sat.max_variable(), 2);
        assert_eq!(sat.num_variables(), 2);
        assert_eq!(sat.num_clauses(), 1);
        assert_eq!(sat.solve(), Some(true));
        assert_eq!(
            sat.solve_with([-1].iter().copied(), iter::empty::<i32>()),
            Some(true)
        );
        assert_eq!(sat.value(1), Some(false));
        assert_eq!(sat.value(-1), Some(true));
        assert_eq!(sat.value(2), Some(true));
        assert_eq!(sat.value(-2), Some(false));
        assert_eq!(
            sat.solve_with([-2].iter().copied(), iter::empty::<i32>()),
            Some(true)
        );
        assert_eq!(sat.value(1), Some(true));
        assert_eq!(sat.value(-1), Some(false));
        assert_eq!(sat.value(2), Some(false));
        assert_eq!(sat.value(-2), Some(true));
        assert_eq!(
            sat.solve_with([-1, -2].iter().copied(), iter::empty::<i32>()),
            Some(false)
        );
        assert_eq!(sat.failed(-1), true);
        assert_eq!(sat.failed(-2), true);
        assert_eq!(sat.status(), Some(false));
        sat.add_clause([4, 5]);
        assert_eq!(sat.status(), None);
        assert_eq!(sat.max_variable(), 5);
        assert_eq!(sat.num_variables(), 4);
        assert_eq!(sat.num_clauses(), 2);
        assert_eq!(
            sat.solve_with([-1, -2, -4].iter().copied(), iter::empty::<i32>()),
            Some(false)
        );
        assert_eq!(sat.failed(-1), true);
        assert_eq!(sat.failed(-2), true);
        assert_eq!(sat.failed(-4), false);
    }

    #[test]
    fn solve_with_temporary_constraint() {
        let mut sat: Solver = Solver::new();
        assert!(sat.signature().starts_with("cadical-"));
        assert_eq!(sat.status(), None);
        sat.add_clause([1, 2]);
        assert_eq!(sat.max_variable(), 2);
        assert_eq!(sat.num_variables(), 2);
        assert_eq!(sat.num_clauses(), 1);
        assert_eq!(sat.solve(), Some(true));
        assert_eq!(
            sat.solve_with([1].iter().copied(), [-1, -2].iter().copied()),
            Some(true)
        );
        assert_eq!(sat.value(1), Some(true));
        assert_eq!(sat.value(2), Some(false));
    }

    fn pigeon_hole(num: i32) -> Solver {
        let mut sat: Solver = Solver::new();
        for i in 0..(num + 1) {
            sat.add_clause((0..num).map(|j| 1 + i * num + j));
        }
        for i1 in 0..(num + 1) {
            for i2 in 0..(num + 1) {
                if i1 == i2 {
                    continue;
                }
                for j in 0..num {
                    let l1 = 1 + i1 * num + j;
                    let l2 = 1 + i2 * num + j;
                    sat.add_clause([-l1, -l2])
                }
            }
        }
        sat
    }

    #[test]
    fn timeout() {
        let mut sat = pigeon_hole(9);
        let started = Instant::now();
        sat.set_callbacks(Some(Timeout::new(0.2)));
        let result = sat.solve();
        let elapsed = started.elapsed().as_secs_f32();
        if result.is_none() {
            assert!(0.1 < elapsed && elapsed < 0.3);
        } else {
            assert!(result == Some(false) && elapsed <= 0.3);
        }

        let started = Instant::now();
        sat.set_callbacks(Some(Timeout::new(0.5)));
        let result = sat.solve();
        let elapsed = started.elapsed().as_secs_f32();
        if result.is_none() {
            assert!(0.4 < elapsed && elapsed < 0.6);
        } else {
            assert!(result == Some(false) && elapsed <= 0.6);
        }

        sat.set_callbacks(None);
        assert_eq!(sat.solve(), Some(false));
    }

    #[test]
    fn decision_limit() {
        let mut sat = pigeon_hole(5);
        sat.set_limit("decisions", 100).unwrap();
        let result = sat.solve();
        assert_eq!(result, None);
        sat.set_limit("decisions", -1).unwrap();
        let result = sat.solve();
        assert_eq!(result, Some(false));
    }

    #[test]
    fn conflict_limit() {
        let mut sat = pigeon_hole(5);
        sat.set_limit("conflicts", 100).unwrap();
        let result = sat.solve();
        assert_eq!(result, None);
        sat.set_limit("conflicts", -1).unwrap();
        let result = sat.solve();
        assert_eq!(result, Some(false));
    }

    #[test]
    fn bad_limit() {
        let mut sat = pigeon_hole(5);
        assert!(sat.set_limit("\0", 0) == Err(Error::new("invalid string")));
        assert!(sat.set_limit("bad", 0) == Err(Error::new("unknown limit")));
    }

    #[test]
    fn moving() {
        let mut sat = pigeon_hole(5);
        let id = thread::spawn(move || {
            assert_eq!(sat.solve(), Some(false));
        });
        id.join().unwrap();
    }

    #[test]
    fn fileio() {
        let mut path = std::env::temp_dir();
        path.push("pigeon5.cnf");

        let mut sat = pigeon_hole(5);
        println!("writing DIMACS to: {path:?}");
        assert!(sat.write_dimacs(&path).is_ok());
        assert!(path.is_file());
        let num_vars = sat.max_variable();

        println!("reading DIMACS from: {path:?}");
        let mut sat: Solver = Default::default();
        assert_eq!(sat.read_dimacs(&path), Ok(num_vars));
        assert_eq!(sat.solve(), Some(false));

        let path = Path::new("MISSINGFILE");
        let mut sat: Solver = Default::default();
        let res = sat.read_dimacs(path);
        assert!(res.is_err());
        println!("reading DIMACS error: {}", res.err().unwrap());
    }
}
