//! Build script for ccadical.
//! This script is responsible for compiling the cadical C++ library.
//! For more information:
//! https://doc.rust-lang.org/cargo/reference/build-scripts.html
//! https://doc.rust-lang.org/cargo/reference/build-script-examples.html

use std::{env, fs, path::Path, process::Command};

fn compile_using_cc() {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .flag_if_supported("-std=c++11")
        .warnings(true)
        .define("NBUILD", None)
        .define("NUNLOCKED", None)
        .define("NTRACING", None)
        .define("QUIET", None);

    let version = std::fs::read_to_string("cadical/VERSION");
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
    let dir_entries = fs::read_dir("cadical/src").unwrap();
    for path in dir_entries {
        let dir_entry = path.unwrap();
        let path = dir_entry.path();
        let path_str = path.to_str().unwrap().to_string();
        if path_str.ends_with(".cpp")
            // mobical should be ignored
            && (!path_str.contains("/mobical.cpp"))
            // added later
            && (!path_str.contains("/resources.cpp"))
            // added later
            && (!path_str.contains("/lookahead.cpp")) 
            // already added in src/ccadical.cpp
            && (!path_str.contains("/ccadical.cpp")) 
            // contains another main function
            && (!path_str.contains("/cadical.cpp"))
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
        files.push("cadical/src/resources.cpp".to_string());
        files.push("cadical/src/lookahead.cpp".to_string());
    }

    // add files which will be compiled
    build.files(files.iter());

    // tell the compiler to recompile if any of the files changed
    for file in files.iter() {
        println!("cargo:rerun-if-changed={file}");
    }

    // compile
    build.compile("ccadical");
}

/// Not ready yet
fn _compile_using_cadical_script() {
    // change working director into cadical
    let cadical_path = Path::new("./cadical");
    let cd_result = env::set_current_dir(cadical_path).is_ok();

    if !cd_result {
        panic!(
            "Failed to change working directory to {}!",
            cadical_path.display()
        );
    }

    // clean previous setup
    let clean_command = "ls";
    let clean_result = Command::new(clean_command)
        .env("PATH", cadical_path)
        .output();
    if let Err(e) = clean_result {
        panic!(
            "Failed to execute CaDiCal clean command: '{}'\nThis error was received: {}",
            clean_command, e
        );
    }

    // run configuration and compilation command
    let command = "./configure && make";
    let comp_result = Command::new(command).output();
    if let Err(e) = comp_result {
        panic!(
            "Failed to execute CaDiCal configuration & compilation command: '{}'\nThis error was received: {}",
            command, e
        );
    }
}

fn main() -> std::io::Result<()> {
    // build Cadical as instructed in it
    // let output = if cfg!(target_os = "windows") {
    //     Command::new("cmd")
    //         .args(["/C", "echo hello"])
    //         .output()
    //         .expect("failed to execute process")
    // } else {

    // };

    //

    // .unwrap_or_else(|_| {
    //     panic!(
    //         "Failed to execute CaDiCal compilation command '{}'",
    //         command_to_compile_cadical
    //     )
    // });

    compile_using_cc();

    Ok(())
}
