//! Build script for ccadical.
//! This script is responsible for compiling the cadical C++ library.
//! For more information:
//! <https://doc.rust-lang.org/cargo/reference/build-scripts.html>
//! <https://doc.rust-lang.org/cargo/reference/build-script-examples.html>

// ************************************************************************************************
// use
// ************************************************************************************************

use std::{env, fs, path::Path, process::Command};

// ************************************************************************************************
// constants
// ************************************************************************************************

const CADICAL_PATH: &str = "cadical";

// ************************************************************************************************
// Compile using cc crate
// ************************************************************************************************

fn _compile_using_cc() {
    let mut build = cc::Build::new();

    // set to c++
    build.cpp(true).flag_if_supported("-std=c++11");

    // disable default flags
    build.no_default_flags(true);

    // add the flags used by cadical 'configure: compiling with 'g++ -Wall -Wextra -O3 -DNDEBUG -DNBUILD'

    // this adds -Wall and -Wextra
    build.warnings(true);

    // define pre compilation variables
    build.define("NDEBUG", None);
    build.define("NBUILD", None);
    build.define("NUNLOCKED", None);
    build.define("NTRACING", None);
    build.define("QUIET", None);

    let version = std::fs::read_to_string(format!("{CADICAL_PATH}/VERSION"));
    let version = version.expect("missing cadical submodule");
    let version = format!("\"{}\"", version.trim());
    build.define("VERSION", version.as_ref());

    // assertions only for debug builds with debug feature enabled
    if std::env::var("PROFILE").unwrap() == "debug"
        && std::env::var("CARGO_FEATURE_CPP_DEBUG").is_ok()
    {
        build.debug(true);
    } else {
        build.debug(false).opt_level(3).define("NDEBUG", None);
    }

    // create list of files to compile
    let mut files = vec![];

    // add interface that we added
    files.push("src/ccadical.cpp".to_string());

    // add cadical .cpp files
    let dir_entries = fs::read_dir(format!("{CADICAL_PATH}/src")).unwrap();
    for path in dir_entries {
        let dir_entry = path.unwrap();
        let path = dir_entry.path();
        let path_str = path.to_str().unwrap().to_string();
        if std::path::Path::new(&path_str)
                     .extension()
                     .map_or(false, |ext| ext.eq_ignore_ascii_case("cpp"))
            // mobical should be ignored
            && (!path_str.ends_with("/mobical.cpp"))
            // added later
            && (!path_str.ends_with("/resources.cpp"))
            // added later
            && (!path_str.ends_with("/lookahead.cpp")) 
            // already added in src/ccadical.cpp
            && (!path_str.ends_with("/ccadical.cpp")) 
            // contains another main function
            && (!path_str.ends_with("/cadical.cpp"))
        {
            // eprintln!("Compiling path {}", path_str);
            files.push(path_str);
        }
    }

    // add resources and lookahead files
    if build.get_compiler().is_like_msvc() {
        build.include(std::path::Path::new("src/msvc"));
        files.push("src/msvc/resources.cpp".to_string());
        files.push("src/msvc/lookahead.cpp".to_string());
    } else {
        files.push(format!("{CADICAL_PATH}/src/resources.cpp"));
        files.push(format!("{CADICAL_PATH}/src/lookahead.cpp"));
    }

    // add files which will be compiled
    build.files(files.iter());

    // tell the compiler to recompile if any of the files changed
    for file in &files {
        println!("cargo:rerun-if-changed={file}");
    }
    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CFLAGS");
    println!("cargo:rerun-if-env-changed=CXX");
    println!("cargo:rerun-if-env-changed=CXXFLAGS");
    println!("cargo:rerun-if-env-changed=CXXSTDLIB");
    println!("cargo:rerun-if-env-changed=CRATE_CC_NO_DEFAULTS");

    // compile
    build.compile("ccadical");
}

// ************************************************************************************************
// Compile using the ./config && make script
// ************************************************************************************************

fn _run_command(command: &mut Command) {
    let command_str = format!("{command:?}");
    match command.output() {
        Ok(_) => println!("cargo:warning=Command {command_str} was successful"),
        Err(e) => {
            panic!("Failed to execute command:\n{}\nERROR:\n{}", command_str, e);
        }
    }
}

fn _change_directory(path: &str) {
    match env::set_current_dir(Path::new(path)) {
        Ok(()) => println!(
            "cargo:warning=Changed working directory to {}",
            env::current_dir().unwrap().display()
        ),
        Err(e) => panic!("Failed to change directory to:\n{}\nERROR:\n{}", path, e),
    }
}

fn _make_dir(dir: &str) {
    match fs::create_dir_all(dir) {
        Ok(()) => println!("cargo:warning=Created directory {dir}"),
        Err(e) => panic!("Failed to create directory:\n{}\nERROR:\n{}", dir, e),
    }
}

/// Not ready yet, mainly there are issues with using cargo clean to clean the build.
/// The problem is that cargo clean will delete the target directory,
/// which will does not delete the cadical build. Both solutions of either performing
/// "make clean" on build or making the script compile into target ran into issues.
fn _compile_using_cadical_script() {
    // always recompile when anything changes
    // println!("cargo:rerun-if-changed=/{}", CADICAL_PATH);

    // change working directory to cadical
    _change_directory(format!("./{CADICAL_PATH}").as_ref());

    // clean previous build
    // _run_command(Command::new("make").arg("clean"));

    // configure makefile
    _run_command(&mut Command::new("./configure"));

    // compile
    _run_command(&mut Command::new("make"));

    panic!();
}

// ************************************************************************************************
// Main build function
// ************************************************************************************************

fn main() {
    _compile_using_cc();
}
