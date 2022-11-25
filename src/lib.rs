//! This is a stand alone crate that contains both the C++ source code of the
//! CaDiCaL incremental SAT solver together with its Rust binding. The C++
//! files are compiled and statically linked during the build process. This
//! crate works on Linux, Apple OSX, Windows, Android, iOS, Raspberry Pi,
//! NetBSD and FreeBSD.
//! CaDiCaL won first place in the SAT track of the SAT Race 2019 and second
//! overall place. It was written by Armin Biere, and it is available under the
//! MIT license.

use std::ffi::{CStr, CString};
use std::mem::ManuallyDrop;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr::null_mut;
use std::time::Instant;
use std::{fmt, slice};

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
        data: *mut c_void,
        cbs: Option<extern "C" fn(*mut c_void) -> c_int>,
    );
    fn ccadical_set_learn(
        ptr: *mut c_void,
        data: *mut c_void,
        max_len: c_int,
        cbs: Option<extern "C" fn(*mut c_void, *const c_int)>,
    );
    fn ccadical_status(ptr: *mut c_void) -> c_int;
    fn ccadical_vars(ptr: *mut c_void) -> c_int;
    fn ccadical_active(ptr: *mut c_void) -> i64;
    fn ccadical_irredundant(ptr: *mut c_void) -> i64;
    fn ccadical_read_dimacs(
        ptr: *mut c_void,
        path: *const c_char,
        vars: *mut c_int,
        strict: c_int,
    ) -> *const c_char;
    fn ccadical_write_dimacs(
        ptr: *mut c_void,
        path: *const c_char,
        min_max_var: c_int,
    ) -> *const c_char;
    fn ccadical_configure(ptr: *mut c_void, name: *const c_char) -> c_int;
    fn ccadical_limit2(ptr: *mut c_void, name: *const c_char, limit: c_int) -> c_int;
}

/// The CaDiCaL incremental SAT solver. The literals are unwrapped positive
/// and negative integers, exactly as in the DIMACS format. The common IPASIR
/// operations are presented in a safe Rust interface.
/// # Examples
/// ```
/// let mut sat: cadical::Solver = Default::default();
/// sat.add_clause([1, 2]);
/// sat.add_clause([-1, 2]);
/// assert_eq!(sat.solve(), Some(true));
/// assert_eq!(sat.value(2), Some(true));
/// ```

pub struct Solver<C: Callbacks = Timeout> {
    ptr: *mut c_void,
    cbs: Option<Box<C>>,
}

impl<C: Callbacks> Solver<C> {
    /// Constructs a new solver instance.
    pub fn new() -> Self {
        let ptr = unsafe { ccadical_init() };
        Self { ptr, cbs: None }
    }

    /// Constructs a new solver with one of the following pre-defined
    /// configurations of advanced internal options:
    /// * `default`: set default advanced internal options
    /// * `plain`: disable all internal preprocessing options
    /// * `sat`: set internal options to target satisfiable instances
    /// * `unsat`: set internal options to target unsatisfiable instances
    pub fn with_config(config: &str) -> Result<Self, Error> {
        let sat: Self = Default::default();
        let config = CString::new(config).map_err(|_| Error::new("invalid string"))?;
        let res = unsafe { ccadical_configure(sat.ptr, config.as_ptr()) };
        if res != 0 {
            Ok(sat)
        } else {
            Err(Error::new("invalid config"))
        }
    }

    /// Returns the name and version of the CaDiCaL library.
    pub fn signature(&self) -> &str {
        let sig = unsafe { CStr::from_ptr(ccadical_signature()) };
        sig.to_str().unwrap_or("invalid")
    }

    /// Adds the given clause to the solver. Negated literals are negative
    /// integers, positive literals are positive ones. All literals must be
    /// non-zero and different from `i32::MIN`.
    #[inline]
    pub fn add_clause<I>(&mut self, clause: I)
    where
        I: IntoIterator<Item = i32>,
    {
        for lit in clause {
            debug_assert!(lit != 0 && lit != std::i32::MIN);
            unsafe { ccadical_add(self.ptr, lit) };
        }
        unsafe { ccadical_add(self.ptr, 0) };
    }

    /// Solves the formula defined by the added clauses. If the formula is
    /// satisfiable, then `Some(true)` is returned. If the formula is
    /// unsatisfiable, then `Some(false)` is returned. If the solver runs out
    /// of resources or was terminated, then `None` is returned.
    pub fn solve(&mut self) -> Option<bool> {
        if let Some(cbs) = &mut self.cbs {
            cbs.as_mut().started();
        }

        let r = unsafe { ccadical_solve(self.ptr) };
        if r == 10 {
            Some(true)
        } else if r == 20 {
            Some(false)
        } else {
            None
        }
    }

    /// Solves the formula defined by the set of clauses under the given
    /// assumptions.
    pub fn solve_with<I>(&mut self, assumptions: I) -> Option<bool>
    where
        I: Iterator<Item = i32>,
    {
        for lit in assumptions {
            debug_assert!(lit != 0 && lit != std::i32::MIN);
            unsafe { ccadical_assume(self.ptr, lit) };
        }
        self.solve()
    }

    /// Returns the status of the solver as returned by the last call to
    /// `solve` or `solve_with`. The state becomes `None` if a new clause
    /// is added.
    #[inline]
    pub fn status(&self) -> Option<bool> {
        let r = unsafe { ccadical_status(self.ptr) };
        if r == 10 {
            Some(true)
        } else if r == 20 {
            Some(false)
        } else {
            None
        }
    }

    /// Returns the value of the given literal in the last solution. The
    /// state of the solver must be `Some(true)`. The returned value is
    /// `None` if the formula is satisfied regardless of the value of the
    /// literal.
    #[inline]
    pub fn value(&self, lit: i32) -> Option<bool> {
        debug_assert!(self.status() == Some(true));
        debug_assert!(lit != 0 && lit != std::i32::MIN);
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
        debug_assert!(self.status() == Some(false));
        debug_assert!(lit != 0 && lit != std::i32::MIN);
        let val = unsafe { ccadical_failed(self.ptr, lit) };
        val == 1
    }

    /// Returns the maximum variable index in the problem as maintained by
    /// the solver.
    /// # Examples
    /// ```
    /// let mut sat: cadical::Solver = Default::default();
    /// sat.add_clause([1, -3]);
    /// assert_eq!(sat.max_variable(), 3);
    /// assert_eq!(sat.num_variables(), 2);
    /// assert_eq!(sat.num_clauses(), 1);
    /// ```
    #[inline]
    pub fn max_variable(&self) -> i32 {
        unsafe { ccadical_vars(self.ptr) }
    }

    /// Returns the number of active variables in the problem. Variables become
    /// active if a clause is added with it. They become inactive, if they
    /// are eliminated or become fixed at the root level.
    #[inline]
    pub fn num_variables(&self) -> i32 {
        unsafe { ccadical_active(self.ptr) as i32 }
    }

    /// Returns the number of active irredundant clauses. Clauses become
    /// inactive if they are satisfied, subsumed or eliminated.
    #[inline]
    pub fn num_clauses(&self) -> usize {
        unsafe { ccadical_irredundant(self.ptr) as usize }
    }

    /// Sets a solver limit with the corresponding name to the given value.
    /// These limits are only valid for the next `solve` or `solve_with` call
    /// and reset to their default values, which disables them.
    /// The following limits are supported:
    /// * `preprocessing`: the number of preprocessing rounds to be performed
    ///    during the search (defaults to `0`).
    /// * `localsearch`: the number of local search rounds to be performed
    ///    during the search (defaults to `0`).
    /// * `terminate`: this value is regularly decremented and aborts the
    ///    solver when it reaches zero (defaults to `0`).
    /// * `conflicts`: decremented when a conflict is detected
    ///    and aborts the solver when it becomes negative (defaults to `-1`).
    /// * `decisions`: decremented when a decision is made
    ///    and aborts the solver when it becomes negative (defaults to `-1`).
    pub fn set_limit(&mut self, name: &str, limit: i32) -> Result<(), Error> {
        let name = CString::new(name).map_err(|_| Error::new("invalid string"))?;
        let valid = unsafe { ccadical_limit2(self.ptr, name.as_ptr(), limit) };
        if valid != 0 {
            Ok(())
        } else {
            Err(Error::new("unknown limit"))
        }
    }

    /// Sets the callbacks to be called while the solver is running.
    /// # Examples
    /// ```
    /// let mut sat: cadical::Solver = Default::default();
    /// sat.add_clause([1, 2]);
    /// sat.set_callbacks(Some(cadical::Timeout::new(0.0)));
    /// assert_eq!(sat.solve(), None);
    /// ```
    pub fn set_callbacks(&mut self, cbs: Option<C>) {
        if let Some(cbs) = cbs {
            if let Some(data) = &mut self.cbs {
                *data.as_mut() = cbs;
            } else {
                self.cbs = Some(Box::new(cbs));
            }
            let data = self.cbs.as_mut().unwrap();
            let max_length = data.max_length();
            let data = data.as_mut() as *mut C as *mut c_void;
            unsafe {
                ccadical_set_terminate(self.ptr, data, Some(Self::terminate_cb));
                ccadical_set_learn(self.ptr, data, max_length, Some(Self::learn_cb));
            }
        } else {
            self.cbs = None;
            let data = null_mut() as *mut c_void;
            unsafe {
                ccadical_set_terminate(self.ptr, data, None);
                ccadical_set_learn(self.ptr, data, 0, None);
            }
        }
    }

    extern "C" fn terminate_cb(data: *mut c_void) -> c_int {
        debug_assert!(!data.is_null());
        let cbs = unsafe { &mut *(data as *mut C) };
        cbs.terminate() as c_int
    }

    extern "C" fn learn_cb(data: *mut c_void, clause: *const c_int) {
        debug_assert!(!data.is_null() && !clause.is_null());

        let mut len: isize = 0;
        while unsafe { clause.offset(len).read() } != 0 {
            len += 1;
        }
        let clause = unsafe { slice::from_raw_parts(clause, len as usize) };
        let clause = ManuallyDrop::new(clause);

        let cbs = unsafe { &mut *(data as *mut C) };
        cbs.learn(&clause);
    }

    /// Returns a mutable reference to the callbacks.
    pub fn get_callbacks(&mut self) -> Option<&mut C> {
        self.cbs.as_mut().map(|a| a.as_mut())
    }

    /// Writes the problem in DIMACS format to the given file.
    pub fn write_dimacs(&mut self, path: &Path) -> Result<(), Error> {
        let path = dimacs_path(path)?;
        let err = unsafe { ccadical_write_dimacs(self.ptr, path.as_ptr(), 0) };
        if err.is_null() {
            Ok(())
        } else {
            Err(dimacs_error(err))
        }
    }

    /// Reads a problem in DIMACS format from the given file. You must call
    /// this function during configuration time, before adding any clauses.
    /// Returns the number of variables as reported by the loader.
    pub fn read_dimacs(&mut self, path: &Path) -> Result<i32, Error> {
        if self.max_variable() != 0 {
            return Err(Error::new("invalid state"));
        }
        let path = dimacs_path(path)?;
        let mut vars: c_int = 0;
        let err =
            unsafe { ccadical_read_dimacs(self.ptr, path.as_ptr(), &mut vars as *mut c_int, 0) };
        if err.is_null() {
            Ok(vars)
        } else {
            Err(dimacs_error(err))
        }
    }
}

fn dimacs_path(path: &Path) -> Result<CString, Error> {
    let path = path.to_str().ok_or_else(|| Error::new("invalid path"))?;
    CString::new(path).map_err(|_| Error::new("invalid path"))
}

fn dimacs_error(err: *const c_char) -> Error {
    let err = unsafe { CStr::from_ptr(err) };
    Error::new(err.to_str().unwrap_or("invalid response"))
}

impl<C: Callbacks> Default for Solver<C> {
    fn default() -> Self {
        Solver::new()
    }
}

impl<C: Callbacks> Drop for Solver<C> {
    fn drop(&mut self) {
        unsafe { ccadical_release(self.ptr) };
    }
}

/// CaDiCaL does not use thread local variables, so it is possible to
/// move it between threads. However it cannot be used queried concurrently
/// (for example getting the value from multiple threads at once), so we
/// do not implement `Sync`.
unsafe impl<C: Callbacks + Send> Send for Solver<C> {}

/// Callbacks trait for finer control.
pub trait Callbacks {
    /// Called when the `solve` method is called.
    #[inline(always)]
    fn started(&mut self) {}

    /// Called by the solver periodically to check if it should terminate.
    #[inline(always)]
    fn terminate(&mut self) -> bool {
        false
    }

    /// Returns the maximum length of clauses to be passed to `learn`. This
    /// methods will be called only once when `set_callbacks` is called.
    #[inline(always)]
    fn max_length(&self) -> i32 {
        0
    }

    /// Called by the solver when a new derived clause is learnt.
    #[allow(unused_variables)]
    #[inline(always)]
    fn learn(&mut self, clause: &[i32]) {}
}

/// Callbacks implementing a simple timeout.
pub struct Timeout {
    pub started: Instant,
    pub timeout: f32,
}

impl Timeout {
    /// Creates a new timeout structure with the given timeout value.
    pub fn new(timeout: f32) -> Self {
        Timeout {
            started: Instant::now(),
            timeout,
        }
    }
}

impl Callbacks for Timeout {
    #[inline(always)]
    fn started(&mut self) {
        self.started = Instant::now();
    }

    #[inline(always)]
    fn terminate(&mut self) -> bool {
        self.started.elapsed().as_secs_f32() >= self.timeout
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Error type for configuration and DIMACS reading and writing errors.
pub struct Error {
    pub msg: String,
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Error {
            msg: msg.to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.msg.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

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
        assert_eq!(sat.solve_with([-1].iter().copied()), Some(true));
        assert_eq!(sat.value(1), Some(false));
        assert_eq!(sat.value(2), Some(true));
        assert_eq!(sat.solve_with([-2].iter().copied()), Some(true));
        assert_eq!(sat.value(1), Some(true));
        assert_eq!(sat.value(2), Some(false));
        assert_eq!(sat.solve_with([-1, -2].iter().copied()), Some(false));
        assert_eq!(sat.failed(-1), true);
        assert_eq!(sat.failed(-2), true);
        assert_eq!(sat.status(), Some(false));
        sat.add_clause([4, 5]);
        assert_eq!(sat.status(), None);
        assert_eq!(sat.max_variable(), 5);
        assert_eq!(sat.num_variables(), 4);
        assert_eq!(sat.num_clauses(), 2);
        assert_eq!(sat.solve_with([-1, -2, -4].iter().copied()), Some(false));
        assert_eq!(sat.failed(-1), true);
        assert_eq!(sat.failed(-2), true);
        assert_eq!(sat.failed(-4), false);
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
        if result == None {
            assert!(0.1 < elapsed && elapsed < 0.3);
        } else {
            assert!(result == Some(false) && elapsed <= 0.3);
        }

        let started = Instant::now();
        sat.set_callbacks(Some(Timeout::new(0.5)));
        let result = sat.solve();
        let elapsed = started.elapsed().as_secs_f32();
        if result == None {
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
        println!("writing DIMACS to: {:?}", path);
        assert!(sat.write_dimacs(&path).is_ok());
        assert!(path.is_file());
        let num_vars = sat.max_variable();

        println!("reading DIMACS from: {:?}", path);
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
