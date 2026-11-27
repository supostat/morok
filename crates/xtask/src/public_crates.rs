use std::io;
use std::path::{Path, PathBuf};

use toml::Value;

use crate::cargo_manifest::CargoManifest;

const SOURCE_DIRECTORY: &str = "src";
const WORKSPACE_MEMBERS: [&str; 2] = ["workspace", "members"];
const PACKAGE_PUBLISH: [&str; 2] = ["package", "publish"];

/// The `src` directories of the members of the workspace at `workspace` that may be published:
/// those whose manifest does not set `publish = false`.
pub(crate) fn source_directories(workspace: &Path) -> io::Result<Vec<PathBuf>> {
    let mut source_directories = Vec::new();
    for member in members(&CargoManifest::of(workspace)?)? {
        let member_directory = workspace.join(member);
        if may_be_published(&CargoManifest::of(&member_directory)?) {
            source_directories.push(member_directory.join(SOURCE_DIRECTORY));
        }
    }
    Ok(source_directories)
}

fn members(workspace_manifest: &CargoManifest) -> io::Result<Vec<String>> {
    let members = workspace_manifest
        .value_at(&WORKSPACE_MEMBERS)
        .and_then(Value::as_array)
        .ok_or_else(|| workspace_manifest.unusable("it lists no members of a workspace"))?;
    members
        .iter()
        .map(|member| {
            member
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| workspace_manifest.unusable("a member of its workspace is not a path"))
        })
        .collect()
}

fn may_be_published(manifest: &CargoManifest) -> bool {
    manifest.value_at(&PACKAGE_PUBLISH) != Some(&Value::Boolean(false))
}
