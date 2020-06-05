fn main() {
    cc::Build::new()
        .define("NBUILD", None)
        .file("src/cadicalc.cpp")
        .file("cadical/src/version.cpp")
        .compile("cadicalc");
}
