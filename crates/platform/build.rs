use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Command;

const SHADER_DIRECTORY: &str = "shaders";
/// The shaders with entry points; `shapes.metal` is included by each of them.
const SHADERS: [&str; 3] = ["rect", "glyph", "effects"];
const XCRUN: &str = "/usr/bin/xcrun";

fn main() {
    println!("cargo::rerun-if-changed={SHADER_DIRECTORY}");
    let output_directory = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    let library = output_directory.join("morok.metallib");
    let mut intermediates = Vec::new();
    for shader in SHADERS {
        let source = PathBuf::from(SHADER_DIRECTORY).join(format!("{shader}.metal"));
        let intermediate = output_directory.join(format!("{shader}.air"));
        xcrun(&[
            OsStr::new("metal"),
            OsStr::new("-c"),
            source.as_os_str(),
            OsStr::new("-o"),
            intermediate.as_os_str(),
        ]);
        intermediates.push(intermediate);
    }
    let mut link = vec![OsStr::new("metallib")];
    link.extend(intermediates.iter().map(|intermediate| intermediate.as_os_str()));
    link.extend([OsStr::new("-o"), library.as_os_str()]);
    xcrun(&link);
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
