//! This is a mockup implementation of the solver to allow testing the memory
//! safety of the crate with `cargo +nightly miri test`.

#![allow(unused_variables)]

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::{null, null_mut};

pub struct Mockup {
    vars: Vec<bool>,
    clauses: i32,
    conflicts: i32,
    decisions: i32,
    status: i32,
    terminate_data: *const c_void,
    terminate_cbs: Option<extern "C" fn(*const c_void) -> c_int>,
}

impl Mockup {
    fn new() -> Self {
        println!("created");
        Self {
            vars: Default::default(),
            clauses: 0,
            conflicts: -1,
            decisions: -1,
            status: 0,
            terminate_data: null_mut(),
            terminate_cbs: None,
        }
    }
}

impl Drop for Mockup {
    fn drop(&mut self) {
        println!("dropped");
    }
}

pub unsafe fn ccadical_signature() -> *const c_char {
    println!("signature");
    "cadical-mockup\0".as_ptr() as *const c_char
}

pub unsafe fn ccadical_init() -> *mut c_void {
    println!("init");
    let mockup = Box::new(Mockup::new());
    Box::into_raw(mockup) as *mut c_void
}

pub unsafe fn ccadical_release(ptr: *mut c_void) {
    println!("release");
    let mockup = Box::from_raw(ptr as *mut Mockup);
    drop(mockup);
}

pub unsafe fn ccadical_add(ptr: *mut c_void, lit: c_int) {
    let mockup = &mut *(ptr as *mut Mockup);
    if lit == 0 {
        mockup.clauses += 1;
    } else {
        let lit = lit.abs();
        if (mockup.vars.len() as i32) < lit {
            mockup.vars.resize(lit as usize, false);
        }
        mockup.vars[(lit - 1) as usize] = true;
    }
}

pub unsafe fn ccadical_assume(ptr: *mut c_void, lit: c_int) {}

pub unsafe fn ccadical_solve(ptr: *mut c_void) -> c_int {
    println!("solve");
    let mockup = &mut *(ptr as *mut Mockup);

    if let Some(cbs) = mockup.terminate_cbs {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1));
            let val = cbs(mockup.terminate_data);
            if val != 0 {
                return 0;
            }
        }
    }

    mockup.status = if mockup.clauses == 0 || mockup.clauses == 2 {
        10
    } else if mockup.conflicts >= 0 || mockup.decisions >= 0 {
        0
    } else {
        20
    };
    mockup.status
}

pub unsafe fn ccadical_val(ptr: *mut c_void, lit: c_int) -> c_int {
    if lit == 2 {
        2
    } else if lit == -2 {
        -2
    } else {
        0
    }
}

pub unsafe fn ccadical_failed(ptr: *mut c_void, lit: c_int) -> c_int {
    0
}

pub unsafe fn ccadical_set_terminate(
    ptr: *mut c_void,
    data: *const c_void,
    cbs: Option<extern "C" fn(*const c_void) -> c_int>,
) {
    let mockup = &mut *(ptr as *mut Mockup);
    mockup.terminate_data = data;
    mockup.terminate_cbs = cbs;
}

pub unsafe fn ccadical_set_learn(
    ptr: *mut c_void,
    data: *const c_void,
    max_len: c_int,
    cbs: Option<extern "C" fn(*const c_void, *const c_int)>,
) {
}

pub unsafe fn ccadical_status(ptr: *mut c_void) -> c_int {
    let mockup = &mut *(ptr as *mut Mockup);
    mockup.status
}

pub unsafe fn ccadical_vars(ptr: *mut c_void) -> c_int {
    let mockup = &mut *(ptr as *mut Mockup);
    mockup.vars.len() as i32
}

pub unsafe fn ccadical_active(ptr: *mut c_void) -> i64 {
    let mockup = &mut *(ptr as *mut Mockup);
    mockup.vars.iter().filter(|v| **v).count() as i64
}

pub unsafe fn ccadical_irredundant(ptr: *mut c_void) -> i64 {
    let mockup = &mut *(ptr as *mut Mockup);
    mockup.clauses as i64
}

pub unsafe fn ccadical_read_dimacs(
    ptr: *mut c_void,
    path: *const c_char,
    vars: *mut c_int,
    strict: c_int,
) -> *const c_char {
    null::<c_char>()
}

pub unsafe fn ccadical_write_dimacs(
    ptr: *mut c_void,
    path: *const c_char,
    min_max_var: c_int,
) -> *const c_char {
    null::<c_char>()
}

pub unsafe fn ccadical_configure(ptr: *mut c_void, name: *const c_char) -> c_int {
    0
}

pub unsafe fn ccadical_limit2(ptr: *mut c_void, name: *const c_char, limit: c_int) -> c_int {
    let mockup = &mut *(ptr as *mut Mockup);
    let name = CStr::from_ptr(name).to_str().unwrap();
    if name == "conflicts" {
        mockup.conflicts = limit;
        1
    } else if name == "decisions" {
        mockup.decisions = limit;
        1
    } else {
        0
    }
}

pub unsafe fn ccadical_reserve(ptr: *mut c_void, min_max_var: c_int) {
    println!("vars");
    let mockup = &mut *(ptr as *mut Mockup);
    if (mockup.vars.len() as i32) < min_max_var {
        mockup.vars.resize(min_max_var as usize, false);
    }
}
