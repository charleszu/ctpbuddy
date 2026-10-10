use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let default_path = manifest_dir.join("../../py/ctpbuddy/contracts/journal_contract.json");
    let source = env::var_os("CTPBUDDY_JOURNAL_CONTRACT")
        .map(PathBuf::from)
        .unwrap_or(default_path);
    println!("cargo:rerun-if-env-changed=CTPBUDDY_JOURNAL_CONTRACT");
    println!("cargo:rerun-if-changed={}", source.display());
    let contents = fs::read_to_string(&source).unwrap_or_else(|error| {
        panic!(
            "cannot read journal contract {}: {error}; set CTPBUDDY_JOURNAL_CONTRACT to the checked-in artifact",
            source.display()
        )
    });
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("journal_contract.json");
    fs::write(out, contents).expect("cannot copy journal contract into OUT_DIR");
    let vectors = env::var_os("CTPBUDDY_JOURNAL_VECTORS")
        .map(PathBuf::from)
        .unwrap_or_else(|| source.with_file_name("journal_vectors.json"));
    println!("cargo:rerun-if-env-changed=CTPBUDDY_JOURNAL_VECTORS");
    println!("cargo:rerun-if-changed={}", vectors.display());
    let vector_contents = fs::read_to_string(&vectors).unwrap_or_else(|error| {
        panic!("cannot read journal vectors {}: {error}", vectors.display())
    });
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("journal_vectors.json"),
        vector_contents,
    )
    .expect("cannot copy journal vectors into OUT_DIR");
}
