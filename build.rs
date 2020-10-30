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

    let mut files = vec![
        "src/ccadical.cpp",
        "cadical/src/version.cpp",
        "cadical/src/solver.cpp",
        "cadical/src/internal.cpp",
        "cadical/src/arena.cpp",
        "cadical/src/proof.cpp",
        "cadical/src/limit.cpp",
        "cadical/src/options.cpp",
        "cadical/src/stats.cpp",
        "cadical/src/message.cpp",
        "cadical/src/external.cpp",
        "cadical/src/profile.cpp",
        "cadical/src/terminal.cpp",
        "cadical/src/clause.cpp",
        "cadical/src/backtrack.cpp",
        "cadical/src/phases.cpp",
        "cadical/src/report.cpp",
        "cadical/src/flags.cpp",
        "cadical/src/solution.cpp",
        "cadical/src/assume.cpp",
        "cadical/src/queue.cpp",
        "cadical/src/checker.cpp",
        "cadical/src/score.cpp",
        "cadical/src/lucky.cpp",
        "cadical/src/propagate.cpp",
        "cadical/src/analyze.cpp",
        "cadical/src/ema.cpp",
        "cadical/src/averages.cpp",
        "cadical/src/minimize.cpp",
        "cadical/src/extend.cpp",
        "cadical/src/restore.cpp",
        "cadical/src/walk.cpp",
        "cadical/src/watch.cpp",
        "cadical/src/decide.cpp",
        "cadical/src/collect.cpp",
        "cadical/src/var.cpp",
        "cadical/src/condition.cpp",
        "cadical/src/occs.cpp",
        "cadical/src/subsume.cpp",
        "cadical/src/elim.cpp",
        "cadical/src/cover.cpp",
        "cadical/src/block.cpp",
        "cadical/src/backward.cpp",
        "cadical/src/vivify.cpp",
        "cadical/src/probe.cpp",
        "cadical/src/decompose.cpp",
        "cadical/src/rephase.cpp",
        "cadical/src/reduce.cpp",
        "cadical/src/gates.cpp",
        "cadical/src/deduplicate.cpp",
        "cadical/src/restart.cpp",
        "cadical/src/ternary.cpp",
        "cadical/src/transred.cpp",
        "cadical/src/instantiate.cpp",
        "cadical/src/bins.cpp",
        "cadical/src/compact.cpp",
        "cadical/src/contract.cpp",
        "cadical/src/util.cpp",
        "cadical/src/config.cpp",
        "cadical/src/file.cpp",
        "cadical/src/tracer.cpp",
        "cadical/src/parse.cpp",
        "cadical/src/format.cpp",
    ];

    if build.get_compiler().is_like_msvc() {
        build.include(std::path::Path::new("src/msvc"));
        files.push("src/msvc/resources.cpp");
        files.push("src/msvc/lookahead.cpp");
    } else {
        files.push("cadical/src/resources.cpp");
        files.push("cadical/src/lookahead.cpp");
    }

    build.files(files.iter());
    for &file in files.iter() {
        println!("cargo:rerun-if-changed={}", file);
    }

    build.compile("ccadical");
    Ok(())
}
