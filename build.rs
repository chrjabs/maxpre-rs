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
    build_maxpre(
        "https://bitbucket.org/coreo-group/maxpre2.git",
        "master",
        "7cd387d2b1ca1f72077b0dacd15321aea31ec77c",
    );

    let out_dir = env::var("OUT_DIR").unwrap();

    println!("cargo:rustc-link-search={}", out_dir);
}

fn build_maxpre(repo: &str, branch: &str, commit: &str) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut maxpre_dir_str = out_dir.clone();
    maxpre_dir_str.push_str("/maxpre");
    let maxpre_dir = Path::new(&maxpre_dir_str);
    if update_repo(maxpre_dir, repo, branch, commit)
        || !Path::new(&out_dir).join("libmaxpre.a").exists()
    {
        // Repo changed, rebuild
        // We specify the build manually here instead of calling make for better portability
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
            .define("GIT_IDENTIFIER", Some(&format!("\"{}\"", commit)[..]))
            .files(src_files)
            .compile("maxpre");
    };

    println!("cargo:rustc-link-lib=static=maxpre");

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-flags=-l dylib=c++");

    #[cfg(not(target_os = "macos"))]
    println!("cargo:rustc-flags=-l dylib=stdc++");
}

/// Returns true if there were changes, false if not
fn update_repo(path: &Path, url: &str, branch: &str, commit: &str) -> bool {
    let mut changed = false;
    let target_oid = git2::Oid::from_str(commit)
        .unwrap_or_else(|e| panic!("Invalid commit hash {}: {}", commit, e));
    // Update repo
    let repo = match git2::Repository::open(path) {
        Ok(repo) => {
            // Check if already at correct commit
            if let Some(oid) = repo.head().unwrap().target_peel() {
                if oid == target_oid {
                    return changed;
                }
            };
            // Check if commit needs to be fetched
            if repo.find_commit(target_oid).is_err() {
                // Fetch repo
                let mut remote = repo.find_remote("origin").unwrap_or_else(|_| {
                    panic!("Expected remote \"origin\" in git repo {:?}", path)
                });
                remote
                    .fetch(&[branch], None, None)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Could not fetch \"origin/{}\" for git repo {:?}: {}",
                            branch, path, e
                        )
                    });
                drop(remote);
            }
            repo
        }
        Err(_) => {
            if path.exists() {
                fs::remove_dir_all(path).unwrap_or_else(|e| {
                    panic!(
                        "Could not delete directory {}: {}",
                        path.to_str().unwrap(),
                        e
                    )
                });
            };
            changed = true;
            git2::Repository::clone(url, path)
                .unwrap_or_else(|e| panic!("Could not clone repository {}: {}", url, e))
        }
    };
    let target_commit = repo
        .find_commit(target_oid)
        .unwrap_or_else(|e| panic!("Could not find commit {}: {}", commit, e));
    repo.checkout_tree(target_commit.as_object(), None)
        .unwrap_or_else(|e| panic!("Could not checkout commit {}: {}", commit, e));
    repo.set_head_detached(target_oid)
        .unwrap_or_else(|e| panic!("Could not detach head at {}: {}", commit, e));
    changed
}
