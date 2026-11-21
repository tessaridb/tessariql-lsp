fn main() {
    println!("cargo:rerun-if-changed=src/parser.c");
    let dir = std::path::Path::new("src");
    let mut build = cc::Build::new();
    build
        .include(dir)
        .file(dir.join("parser.c"))
        .warnings(false)
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-but-set-variable");
    #[cfg(target_env = "msvc")]
    build.flag("/utf-8");
    build.compile("tree-sitter-tessariql");
}
