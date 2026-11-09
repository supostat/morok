use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

const SHADERS: &str = "shaders/rect.metal";
const XCRUN: &str = "/usr/bin/xcrun";

fn main() {
    println!("cargo::rerun-if-changed={SHADERS}");
    let output_directory = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    let intermediate = output_directory.join("rect.air");
    let library = output_directory.join("morok.metallib");
    xcrun(&[
        OsStr::new("metal"),
        OsStr::new("-c"),
        OsStr::new(SHADERS),
        OsStr::new("-o"),
        intermediate.as_os_str(),
    ]);
    xcrun(&[
        OsStr::new("metallib"),
        intermediate.as_os_str(),
        OsStr::new("-o"),
        library.as_os_str(),
    ]);
    println!("cargo::rustc-env=MOROK_SHADERS_METALLIB={}", library.display());
}

fn xcrun(tool_with_arguments: &[&OsStr]) {
    let status = Command::new(XCRUN)
        .args(["-sdk", "macosx"])
        .args(tool_with_arguments)
        .status()
        .unwrap_or_else(|error| panic!("{XCRUN} runs: {error}"));
    assert!(
        status.success(),
        "{XCRUN} -sdk macosx {tool_with_arguments:?} failed; the shaders need the Metal toolchain of Xcode"
    );
}
