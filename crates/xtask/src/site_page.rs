use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use toml::Value;

use crate::cargo_manifest::CargoManifest;
use crate::finding::Finding;
use crate::frame_files::FrameFiles;
use crate::line_number;
use crate::png_size::PngSize;
use crate::start_tag::StartTag;

const SITE_DIRECTORY: &str = "site";
const PAGE: &str = "index.html";
const FRAME_LIST: &str = "frames.toml";
const REPOSITORY: [&str; 3] = ["workspace", "package", "repository"];
/// What follows the address of the repository in a link to a file of its main branch.
const FILES_OF_MAIN_BRANCH: &str = "/blob/main/";

const PICTURE: &str = "img";
const SCRIPT: &str = "script";
const LINK: &str = "link";
const SOURCE: &str = "src";
const HREF: &str = "href";
const WIDTH: &str = "width";
const HEIGHT: &str = "height";
const SCHEME_END: char = ':';
const SCHEME_MARKS: [char; 3] = ['+', '-', '.'];
const PLACE_OF_THE_PAGE: char = '#';

const NO_PAGE: &str = "the page of the site is missing";
const NO_REPOSITORY: &str = "it names no repository of the workspace";
const PICTURE_FROM_ELSEWHERE: &str = "the picture names no file of the site: a picture of the page is a frame the list of \
                                      frames names";

/// What the tags of the page of a site are held against.
struct PageRules<'workspace> {
    workspace: &'workspace Path,
    site_directory: PathBuf,
    page_file: PathBuf,
    /// None when the list does not read: its own finding stands for every picture then.
    frame_files: Option<FrameFiles>,
    /// What a link to a file of the repository of the workspace starts with.
    repository_files: String,
}

/// Reports whatever keeps the page of the site of the workspace at `workspace` whole: the page
/// and its list of frames, the files the page asks its folder for, its pictures against the
/// list and against their own sizes, and its links to files of the repository.
pub(crate) fn findings(workspace: &Path) -> io::Result<Vec<Finding>> {
    let site_directory = workspace.join(SITE_DIRECTORY);
    let (frame_files, mut findings) = match FrameFiles::read(&site_directory.join(FRAME_LIST))? {
        Ok(frame_files) => {
            let unwritten = frame_files.unwritten();
            (Some(frame_files), unwritten)
        }
        Err(unread_list) => (None, vec![unread_list]),
    };
    let page_file = site_directory.join(PAGE);
    match fs::read_to_string(&page_file) {
        Ok(page_text) => {
            let rules = PageRules {
                workspace,
                site_directory,
                page_file,
                frame_files,
                repository_files: repository_files(workspace)?,
            };
            for start_tag in StartTag::all_in(&page_text) {
                findings.extend(rules.findings_of(&start_tag)?);
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => findings.push(Finding {
            path: page_file,
            line: line_number::FIRST,
            reason: NO_PAGE.to_owned(),
        }),
        Err(error) => return Err(error),
    }
    Ok(findings)
}

impl PageRules<'_> {
    fn findings_of(&self, start_tag: &StartTag<'_>) -> io::Result<Vec<Finding>> {
        let reasons = [
            self.reason_against_reference(start_tag)?,
            self.reason_against_link(start_tag),
        ];
        Ok(reasons
            .into_iter()
            .flatten()
            .map(|reason| Finding {
                path: self.page_file.clone(),
                line: start_tag.line,
                reason,
            })
            .collect())
    }

    /// Why the folder cannot serve what the tag asks it for; none when it can, or when the tag
    /// asks it for nothing.
    fn reason_against_reference(&self, start_tag: &StartTag<'_>) -> io::Result<Option<String>> {
        let is_picture = start_tag.opens(PICTURE);
        let Some(reference) = local_reference_of(start_tag) else {
            return Ok(is_picture.then(|| PICTURE_FROM_ELSEWHERE.to_owned()));
        };
        if !names_a_file_under_its_directory(Path::new(reference)) {
            return Ok(Some(format!(
                "`{reference}` is no path under the folder of the site, which is all that is published"
            )));
        }
        if !self.site_directory.join(reference).is_file() {
            return Ok(Some(format!("the folder of the site holds no file `{reference}`")));
        }
        if is_picture {
            self.reason_against_picture(start_tag, reference)
        } else {
            Ok(None)
        }
    }

    /// Why the file of the folder is not the picture its tag makes of it.
    fn reason_against_picture(&self, picture: &StartTag<'_>, file: &str) -> io::Result<Option<String>> {
        let is_a_listed_frame = self
            .frame_files
            .as_ref()
            .is_none_or(|frame_files| frame_files.names(Path::new(file)));
        if !is_a_listed_frame {
            return Ok(Some(format!(
                "the picture `{file}` is no frame the list of frames names: the pictures of the page are what the \
                 stand writes from the list"
            )));
        }
        let Some(size) = PngSize::of(&self.site_directory.join(file))? else {
            return Ok(Some(format!("the picture `{file}` is no PNG file")));
        };
        let declares = |attribute: &str, pixels: u32| picture.attribute(attribute) == Some(pixels.to_string().as_str());
        if declares(WIDTH, size.width) && declares(HEIGHT, size.height) {
            Ok(None)
        } else {
            Ok(Some(format!(
                "the tag of the picture `{file}` does not declare the size of its file: `width=\"{}\" height=\"{}\"`",
                size.width, size.height
            )))
        }
    }

    /// Why a link of the tag to a file of the repository leads nowhere.
    fn reason_against_link(&self, start_tag: &StartTag<'_>) -> Option<String> {
        let linked_file = start_tag.attribute(HREF)?.strip_prefix(&self.repository_files)?;
        let is_held = self.workspace.join(linked_file).is_file();
        (!is_held).then(|| format!("the link names `{linked_file}`, a file the workspace does not hold"))
    }
}

fn repository_files(workspace: &Path) -> io::Result<String> {
    let manifest = CargoManifest::of(workspace)?;
    let repository = manifest
        .value_at(&REPOSITORY)
        .and_then(Value::as_str)
        .ok_or_else(|| manifest.unusable(NO_REPOSITORY))?;
    Ok(format!("{repository}{FILES_OF_MAIN_BRANCH}"))
}

/// What the tag asks the folder of the page for: the `src` of a picture or a script and the
/// `href` of a `link`, unless the value carries a scheme or names a place of the page itself.
fn local_reference_of<'page>(start_tag: &StartTag<'page>) -> Option<&'page str> {
    let attribute = if start_tag.opens(PICTURE) || start_tag.opens(SCRIPT) {
        SOURCE
    } else if start_tag.opens(LINK) {
        HREF
    } else {
        return None;
    };
    start_tag
        .attribute(attribute)
        .filter(|reference| !has_scheme(reference) && !reference.starts_with(PLACE_OF_THE_PAGE))
}

fn has_scheme(reference: &str) -> bool {
    reference.split_once(SCHEME_END).is_some_and(|(scheme, _)| {
        scheme.starts_with(|first: char| first.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || SCHEME_MARKS.contains(&character))
    })
}

fn names_a_file_under_its_directory(file: &Path) -> bool {
    let mut components = file.components().peekable();
    components.peek().is_some() && components.all(|component| matches!(component, Component::Normal(_)))
}
