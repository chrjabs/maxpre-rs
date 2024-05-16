#![allow(dead_code, unused)]
use glob::glob;
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
    process::Command,
    str,
};

fn main() {
    if std::env::var("DOCS_RS").is_ok() {
        // don't build c++ library on docs.rs due to network restrictions
        return;
    }

    // Build C++ library
    build();

    let out_dir = env::var("OUT_DIR").unwrap();

    println!("cargo:rerun-if-changed=cppsrc/");

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-flags=-l dylib=c++");

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    println!("cargo:rustc-flags=-l dylib=stdc++");

    // Built solver is in out_dir
    println!("cargo:rustc-link-search={}", out_dir);
    println!("cargo:rustc-link-search={}/lib", out_dir);
}

fn build() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut maxpre_dir_str = crate_dir.clone();
    maxpre_dir_str.push_str("/cppsrc");
    let maxpre_dir = Path::new(&maxpre_dir_str);

    // Specify the build manually here instead of calling make for better portability
    let src_files = vec![
        "preprocessor.cpp",
        "preprocessedinstance.cpp",
        "trace.cpp",
        "utility.cpp",
        "probleminstance.cpp",
        "timer.cpp",
        "clause.cpp",
        "log.cpp",
        "AMSLEX.cpp",
        "touchedlist.cpp",
        "preprocessorinterface.cpp",
        "cardinalityconstraint.cpp",
        "satlikeinterface.cpp",
        "cpreprocessorinterface.cpp",
        "prooflogger.cpp",
        "satsolver/solvers/glucose3/utils/System.cc",
        "satsolver/solvers/glucose3/core/Solver.cc",
    ]
    .into_iter()
    .map(|sf| maxpre_dir.join("src").join(sf));

    // Setup build
    let mut build = cc::Build::new();
    build.cpp(true);
    if env::var("PROFILE").unwrap() == "debug" {
        build
            .opt_level(0)
            .define("DEBUG", None)
            .warnings(true)
            .debug(true);
    } else {
        build.opt_level(3).define("NDEBUG", None).warnings(false);
    };

    // Build MaxPre
    build
        .include(maxpre_dir.join("src"))
        .include(maxpre_dir.join("src/satsolver/solvers/glucose3"))
        .define("GIT_IDENTIFIER", Some("\"maxpre-rs build\""))
        .files(src_files)
        .compile("maxpre");
}
