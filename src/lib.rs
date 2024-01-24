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
    // ********************************************************************************************
    // Since CaDiCal is written in C++, and rust bindings are easier to write for c, we
    // use the C wrapper that CaDiCal provides. It is available in 'cadical/src/ccadical.h'
    // ********************************************************************************************
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
    fn ccadical_constrain(ptr: *mut c_void, lit: c_int);
    fn ccadical_constraint_failed(ptr: *mut c_void) -> c_int;
    fn ccadical_status(ptr: *mut c_void) -> c_int;
    fn ccadical_active(ptr: *mut c_void) -> i64;
    fn ccadical_irredundant(ptr: *mut c_void) -> i64;
    fn ccadical_set_option(ptr: *mut c_void, name: *const c_char, val: c_int) -> c_int;
    fn ccadical_simplify(ptr: *mut c_void) -> c_int;
    fn ccadical_freeze(ptr: *mut c_void, lit: c_int);
    // ********************************************************************************************
    // The following functions are c++ functions that we translated into c++ in ccadical.cpp
    // int ccadical_status(CCaDiCaL *wrapper)
    // ********************************************************************************************
    fn ccadical_vars(ptr: *mut c_void) -> c_int;
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

    /// set options for the solver, see ccadical.h for more info
    pub fn set(&mut self, name: &str, val: i32) -> Result<(), Error> {
        let name = CString::new(name).map_err(|_| Error::new("invalid string"))?;
        let valid = unsafe { ccadical_set_option(self.ptr, name.as_ptr(), val) };
        if valid != 0 {
            Ok(())
        } else {
            Err(Error::new("Unknown option."))
        }
    }

    /// This function executes 3 preprocessing rounds. It is
    /// similar to 'solve' with 'limits ("preprocessing", rounds)' except that
    /// no CDCL nor local search, nor lucky phases are executed.  The result
    /// values are also the same:
    /// 1. None=unknown
    /// 2. Some(true)=satisfiable
    /// 3. Some(false)=unsatisfiable
    /// As 'solve' it resets current assumptions and limits before returning.
    ///
    ///   require (READY)
    ///   ensure (UNKNOWN | SATISFIED | UNSATISFIED)
    ///
    pub fn simplify(&mut self) -> Option<bool> {
        let r = unsafe { ccadical_simplify(self.ptr) };
        if r == 10 {
            Some(true)
        } else if r == 20 {
            Some(false)
        } else {
            None
        }
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

    /// We have the following common reference counting functions, which avoid
    /// to restore clauses but require substantial user guidance.  This was the
    /// only way to use inprocessing in incremental SAT solving in Lingeling
    /// (and before in MiniSAT's 'freeze' / 'thaw') and which did not use
    /// automatic clause restoring.  In general this is slower than
    /// restoring clauses and should not be used.
    ///
    /// In essence the user freezes variables which potentially are still
    /// needed in clauses added or assumptions used after the next 'solve'
    /// call.  As in Lingeling you can freeze a variable multiple times, but
    /// then have to melt it the same number of times again in order to enable
    /// variable eliminating on it etc.  The arguments can be literals
    /// (negative indices) but conceptually variables are frozen.
    ///
    /// In the old way of doing things without restore you should not use a
    /// variable incrementally (in 'add' or 'assume'), which was used before
    /// and potentially could have been eliminated in a previous 'solve' call.
    /// This can lead to spurious satisfying assignment.  In order to check
    /// this API contract one can use the 'checkfrozen' option.  This has the
    /// drawback that restoring clauses implicitly would fail with a fatal
    /// error message even if in principle the solver could just restore
    /// clauses. Thus this option is disabled by default.
    ///
    /// See our SAT'19 paper [FazekasBiereScholl-SAT'19] for more details.
    ///
    ///   require (VALID)
    ///   ensure (VALID)
    ///
    #[inline]
    pub fn freeze(&mut self, lit: i32) {
        debug_assert!(lit != 0 && lit != std::i32::MIN);
        unsafe { ccadical_freeze(self.ptr, lit) };
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
    /// assumptions and the clause under the given temporary constraint.
    pub fn solve_with<I, U>(&mut self, assumptions: I, constraint: U) -> Option<bool>
    where
        I: IntoIterator<Item = i32>,
        U: IntoIterator<Item = i32>,
    {
        // add all the assumptions
        for lit in assumptions {
            debug_assert!(lit != 0 && lit != std::i32::MIN);
            unsafe { ccadical_assume(self.ptr, lit) };
        }

        // add the assumed clause and then finalize if needed.
        let mut iterations = 0;
        for lit in constraint {
            iterations += 1;
            debug_assert!(lit != 0 && lit != std::i32::MIN);
            unsafe { ccadical_constrain(self.ptr, lit) };
        }
        // finalize the clause if needed
        if iterations > 0 {
            unsafe { ccadical_constrain(self.ptr, 0) };
        }

        // call the solve function
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
    pub fn value(&mut self, lit: i32) -> Option<bool> {
        debug_assert!(self.status() == Some(true));
        debug_assert!(lit != 0 && lit != std::i32::MIN);
        let val = unsafe { ccadical_val(self.ptr, lit) };
        if val == lit.abs() {
            Some(true)
        } else if val == -lit.abs() {
            Some(false)
        } else {
            None
        }
    }

    /// Checks if the given assumed literal (passed to `solve_with`) was used
    /// in the proof of the unsatisfiability of the formula. The state of the
    /// solver must be `Some(false)`.
    #[inline]
    pub fn failed(&mut self, lit: i32) -> bool {
        debug_assert!(self.status() == Some(false));
        debug_assert!(lit != 0 && lit != std::i32::MIN);
        let val = unsafe { ccadical_failed(self.ptr, lit) };
        val == 1
    }

    /// Checks if the given constraint clause (passed to `solve_with`) was used
    /// in the proof of the unsatisfiability of the formula. The state of the
    /// solver must be `Some(false)`.
    #[inline]
    pub fn constraint_failed(&mut self) -> bool {
        debug_assert!(self.status() == Some(false));
        // debug_assert!(lit != 0 && lit != std::i32::MIN);
        let val = unsafe { ccadical_constraint_failed(self.ptr) };
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
    pub fn max_variable(&mut self) -> i32 {
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

/// `CaDiCaL` does not use thread local variables, so it is possible to
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
    #[must_use]
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
    #[must_use]
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
