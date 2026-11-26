use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use toml::{Table, Value};

const MANIFEST: &str = "Cargo.toml";
const SOURCE_DIRECTORY: &str = "src";

/// The `src` directories of the members of the workspace at `workspace` that may be published:
/// those whose manifest does not set `publish = false`.
pub(crate) fn source_directories(workspace: &Path) -> io::Result<Vec<PathBuf>> {
    let mut source_directories = Vec::new();
    for member in members(&workspace.join(MANIFEST))? {
        let member_directory = workspace.join(member);
        if may_be_published(&manifest(&member_directory.join(MANIFEST))?) {
            source_directories.push(member_directory.join(SOURCE_DIRECTORY));
        }
    }
    Ok(source_directories)
}

fn members(workspace_manifest: &Path) -> io::Result<Vec<String>> {
    let manifest = manifest(workspace_manifest)?;
    let members = manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
        .ok_or_else(|| unusable(workspace_manifest, "it lists no members of a workspace"))?;
    members
        .iter()
        .map(|member| {
            member
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| unusable(workspace_manifest, "a member of its workspace is not a path"))
        })
        .collect()
}

fn may_be_published(manifest: &Table) -> bool {
    let publish = manifest.get("package").and_then(|package| package.get("publish"));
    publish != Some(&Value::Boolean(false))
}

fn manifest(path: &Path) -> io::Result<Table> {
    fs::read_to_string(path)
        .map_err(|error| io::Error::new(error.kind(), format!("{}: {error}", path.display())))?
        .parse()
        .map_err(|error: toml::de::Error| unusable(path, &error.to_string()))
}

fn unusable(manifest: &Path, problem: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, format!("{}: {problem}", manifest.display()))
}
