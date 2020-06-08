use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

extern "C" {
    fn ccadical_signature() -> *const c_char;
    fn ccadical_init() -> *mut c_void;
    fn ccadical_release(ptr: *mut c_void);
    fn ccadical_add(ptr: *mut c_void, lit: c_int);
    fn ccadical_assume(ptr: *mut c_void, lit: c_int);
    fn ccadical_solve(ptr: *mut c_void) -> c_int;
    fn ccadical_val(ptr: *mut c_void, lit: c_int) -> c_int;
    fn ccadical_failed(ptr: *mut c_void, lit: c_int) -> c_int;
    fn ccadical_set_terminate(
        ptr: *mut c_void,
        flag: *const c_void,
        cb: extern "C" fn(*const c_void) -> c_int,
    );
    fn ccadical_print_statistics(ptr: *mut c_void);
}

extern "C" fn terminate_cb(flag: *const c_void) -> c_int {
    let flag = flag as *const AtomicBool;
    let flag = unsafe { &*flag };
    if flag.load(Ordering::Relaxed) {
        1
    } else {
        0
    }
}

/// The CaDiCaL incremental SAT solver.
pub struct Solver {
    ptr: *mut c_void,
    state: Option<bool>,
    terminate: Option<Arc<AtomicBool>>,
}

impl Solver {
    /// Returns the name and version of the CaDiCaL library.
    pub fn signature() -> &'static str {
        let s = unsafe { CStr::from_ptr(ccadical_signature()) };
        s.to_str().unwrap_or("invalid")
    }

    /// Constructs a new solver instance.
    pub fn new() -> Self {
        let ptr = unsafe { ccadical_init() };
        Self {
            ptr,
            state: None,
            terminate: None,
        }
    }

    /// Adds the given clause to the solver. Negated literals are negative
    /// integers, positive literals are positive ones. All literals must be
    /// non-zero and different from `i32::MIN`.
    #[inline]
    pub fn add_clause<I>(&mut self, clause: I)
    where
        I: Iterator<Item = i32>,
    {
        for lit in clause {
            debug_assert!(lit != 0 && lit != i32::MIN);
            unsafe { ccadical_add(self.ptr, lit) };
        }
        unsafe { ccadical_add(self.ptr, 0) };
        self.state = None;
    }

    /// Solves the formula defined by the added clauses. If the formula is
    /// satisfiable, then `Some(true)` is returned. If the formula is
    /// unsatisfiable, then `Some(false)` is returned. If the solver runs out
    /// of resources or was terminated, then `None` is returned.
    pub fn solve(&mut self) -> Option<bool> {
        let r = unsafe { ccadical_solve(self.ptr) };
        self.state = if r == 10 {
            Some(true)
        } else if r == 20 {
            Some(false)
        } else {
            None
        };
        self.state
    }

    /// Solves the formula defined by the set of clauses under the given
    /// assumptions.
    pub fn solve_with<I>(&mut self, assumptions: I) -> Option<bool>
    where
        I: Iterator<Item = i32>,
    {
        for lit in assumptions {
            debug_assert!(lit != 0 && lit != i32::MIN);
            unsafe { ccadical_assume(self.ptr, lit) };
        }
        self.solve()
    }

    /// Returns the state of the solver as returned by the last call to
    /// `solve` or `solve_with`. The state becomes `None` if a new clause
    /// is added.
    #[inline]
    pub fn state(&self) -> Option<bool> {
        self.state
    }

    /// Returns the value of the given literal in the last solution. The
    /// state of the solver must be `Some(true)`. The returned value is
    /// `None` if the formula is satisfied regardless of the the value of the
    /// literal.
    #[inline]
    pub fn value(&self, lit: i32) -> Option<bool> {
        debug_assert!(self.state == Some(true));
        debug_assert!(lit != 0 && lit != i32::MIN);
        let val = unsafe { ccadical_val(self.ptr, lit) };
        if val == lit {
            Some(true)
        } else if val == -lit {
            Some(false)
        } else {
            None
        }
    }

    /// Checks if the given assumed literal (passed to `solve_with`) was used
    /// in the proof of the unsatisfiability of the formula. The state of the
    /// solver must be `Some(false)`.
    #[inline]
    pub fn failed(&self, lit: i32) -> bool {
        debug_assert!(self.state == Some(false));
        debug_assert!(lit != 0 && lit != i32::MIN);
        let val = unsafe { ccadical_failed(self.ptr, lit) };
        val == 1
    }

    /// Returns a flag that can be set asynchronously to terminate the solver
    /// at any time. If this flag is set, then it should be cleared before
    /// `solve` is called again, otherwise it will terminate immediately.
    pub fn terminate_flag(&mut self) -> Arc<AtomicBool> {
        if self.terminate.is_none() {
            self.terminate = Some(Arc::new(AtomicBool::new(false)));
            let flag = self.terminate.as_mut().unwrap();
            let flag = flag.as_ref() as *const AtomicBool as *const c_void;
            unsafe { ccadical_set_terminate(self.ptr, flag, terminate_cb) };
        }
        self.terminate.as_mut().unwrap().clone()
    }

    /// Prints out statistics about the solver.
    pub fn print_statistics(&mut self) {
        unsafe { ccadical_print_statistics(self.ptr) };
    }
}

impl Default for Solver {
    fn default() -> Self {
        Solver::new()
    }
}

impl Drop for Solver {
    fn drop(&mut self) {
        unsafe { ccadical_release(self.ptr) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn solver() {
        assert!(Solver::signature().starts_with("cadical-"));
        let mut sat = Solver::new();
        sat.add_clause([1, 2].iter().copied());
        assert_eq!(sat.solve(), Some(true));
        assert_eq!(sat.solve_with([-1].iter().copied()), Some(true));
        assert_eq!(sat.value(1), Some(false));
        assert_eq!(sat.value(2), Some(true));
        assert_eq!(sat.solve_with([-2].iter().copied()), Some(true));
        assert_eq!(sat.value(1), Some(true));
        assert_eq!(sat.value(2), Some(false));
        assert_eq!(sat.solve_with([-1, -2].iter().copied()), Some(false));
        assert_eq!(sat.failed(-1), true);
        assert_eq!(sat.failed(-2), true);
    }

    fn pigeon_hole(num: i32) -> Solver {
        let mut sat = Solver::new();
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
                    sat.add_clause([-l1, -l2].iter().copied())
                }
            }
        }
        sat
    }

    #[test]
    fn terminate() {
        let mut sat = pigeon_hole(10);
        let flag = sat.terminate_flag();
        assert_eq!(flag.load(Ordering::Relaxed), false);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            flag.store(true, Ordering::Relaxed);
        });
        assert_eq!(sat.solve(), None);
        assert_eq!(sat.terminate_flag().load(Ordering::Relaxed), true);
        sat.print_statistics();
    }
}
