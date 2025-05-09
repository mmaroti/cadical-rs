fn main() -> std::io::Result<()> {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .warnings(false)
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

    let excluded = [
        "cadical/src/ccadical.cpp",
        "cadical/src/cadical.cpp",
        "cadical/src/mobical.cpp",
    ];

    let mut files = vec!["src/ccadical.cpp".to_string()];

    for file in std::fs::read_dir("cadical/src/").unwrap() {
        let file = file.unwrap().path().to_str().unwrap().to_string();
        if file.ends_with(".cpp") && !excluded.contains(&file.as_str()) {
            files.push(file);
        }
    }

    build.files(files.iter());
    for file in files.iter() {
        println!("cargo:rerun-if-changed={}", file);
    }

    println!("cargo:rerun-if-env-changed=CC");
    println!("cargo:rerun-if-env-changed=CFLAGS");
    println!("cargo:rerun-if-env-changed=CXX");
    println!("cargo:rerun-if-env-changed=CXXFLAGS");
    println!("cargo:rerun-if-env-changed=CXXSTDLIB");

    // fixing errors when using clang
    if build.get_compiler().is_like_clang() {
        build.cpp_set_stdlib("c++");
    }

    build.compile("ccadical");
    Ok(())
}
