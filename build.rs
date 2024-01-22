//! Build script for ccadical.
//! This script is responsible for compiling the cadical C++ library.
//! For more information:
//! https://doc.rust-lang.org/cargo/reference/build-scripts.html
//! https://doc.rust-lang.org/cargo/reference/build-script-examples.html

use std::fs;

fn main() -> std::io::Result<()> {
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
        if path_str.ends_with(".cpp") {
            println!("file {}", path_str);
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
    Ok(())
}
