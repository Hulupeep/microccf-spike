use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn cargo_manifest_has_no_ccf_core_dependency() {
    let manifest = fs::read_to_string("Cargo.toml").expect("read Cargo.toml");
    let forbidden = ["ccf-core", "ccf_core"];
    for needle in forbidden {
        assert!(
            !manifest.contains(needle),
            "microCCF spike must not depend on {needle}"
        );
    }
}

#[test]
fn source_does_not_import_or_implement_canonical_ccf_terms() {
    let src_files = rust_sources(Path::new("src")).expect("read src tree");
    let forbidden = [
        "ccf_core",
        "ccf-core",
        "QAC",
        "qac",
        "kappa",
        "Sinkhorn",
        "sinkhorn",
        "min_cut",
        "min-cut",
        "trust dynamics",
    ];

    for file in src_files {
        let body = fs::read_to_string(&file).expect("read Rust source");
        for needle in forbidden {
            assert!(
                !body.contains(needle),
                "{} contains forbidden canonical CCF term: {needle}",
                file.display()
            );
        }
    }
}

fn rust_sources(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(rust_sources(&path)?);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(files)
}
