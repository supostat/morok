use std::path::{Path, PathBuf};

const CRATES_DIRECTORY: &str = "crates";

pub(crate) fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask lives two levels below the workspace root")
        .to_path_buf()
}

pub(crate) fn crates_directory() -> PathBuf {
    root().join(CRATES_DIRECTORY)
}
