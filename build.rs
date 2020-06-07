use std::io::Result;

fn main() -> Result<()> {
    // let version = std::fs::read_to_string("cadical/VERSION")?;

    let files = [
        "cadical/src/ccadical.cpp",
        "src/ccadical_ext.cpp",
        "cadical/src/version.cpp",
        "cadical/src/solver.cpp",
        "cadical/src/internal.cpp",
        "cadical/src/arena.cpp",
        "cadical/src/proof.cpp",
        "cadical/src/limit.cpp",
        "cadical/src/options.cpp",
        "cadical/src/stats.cpp",
        "cadical/src/message.cpp",
        "cadical/src/resources.cpp",
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
    ];

    cc::Build::new()
        .cpp(true)
        .warnings(true)
        .define("NBUILD", None)
        .files(files.iter())
        .compile("ccadical");
    Ok(())
}
