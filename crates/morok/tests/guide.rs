use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use morok::components::KIT_TOKENS;
use morok_css::{SourceFile, compile_stylesheets, compile_tokens};

const WORKSPACE_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const GUIDE_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/docs");
const GUIDE_MODULE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/guide.rs");

const OVERVIEW: &str = "overview.md";
const INDEX: &str = "index.md";
/// The tables of the index: the module the heading of each names, and the source file whose
/// exports the table lists.
const INDEX_TABLES: [(&str, &str); 3] = [
    ("`morok`", "crates/morok/src/lib.rs"),
    ("`morok::components`", "crates/components/src/lib.rs"),
    ("`morok::components::view`", "crates/components/src/view.rs"),
];

const FENCE: &str = "```";
const STYLESHEET_LANGUAGE: &str = "css";
const IGNORED: &str = "ignore";
/// What opens a path of the repository, given from its root.
const REPOSITORY_DIRECTORIES: [&str; 3] = ["crates/", "design/", "docs/"];
/// What opens the two kinds of item that export a name, at the start of a line.
const REEXPORT: &str = "\npub use ";
const TYPE_ALIAS: &str = "\npub type ";

/// A file of the guide.
struct Chapter {
    file_name: String,
    text: String,
}

/// A fenced block of a chapter: the tags after its opening fence, and what it holds.
struct FencedBlock {
    tags: Vec<String>,
    body: String,
    /// The line of the chapter its opening fence stands on.
    line: usize,
}

impl Chapter {
    fn all() -> Vec<Self> {
        let mut chapters: Vec<Self> = fs::read_dir(GUIDE_DIRECTORY)
            .expect("the directory of the guide is readable")
            .map(|entry| {
                let path = entry.expect("an entry of the guide is readable").path();
                Self {
                    file_name: path
                        .file_name()
                        .expect("a file of the guide has a name")
                        .to_string_lossy()
                        .into_owned(),
                    text: fs::read_to_string(&path).expect("a chapter is readable"),
                }
            })
            .collect();
        chapters.sort_by(|left, right| left.file_name.cmp(&right.file_name));
        chapters
    }

    fn named(file_name: &str) -> Self {
        Self::all()
            .into_iter()
            .find(|chapter| chapter.file_name == file_name)
            .unwrap_or_else(|| panic!("the guide has the chapter {file_name}"))
    }

    fn fenced_blocks(&self) -> Vec<FencedBlock> {
        let mut blocks = Vec::new();
        let mut open: Option<FencedBlock> = None;
        for (index, line) in self.text.lines().enumerate() {
            let fence_tags = line.trim_start().strip_prefix(FENCE);
            match (open.take(), fence_tags) {
                (Some(block), Some(_)) => blocks.push(block),
                (Some(mut block), None) => {
                    block.body.push_str(line);
                    block.body.push('\n');
                    open = Some(block);
                }
                (None, Some(tags)) => {
                    open = Some(FencedBlock {
                        tags: tags
                            .split([',', ' '])
                            .filter(|tag| !tag.is_empty())
                            .map(str::to_owned)
                            .collect(),
                        body: String::new(),
                        line: index + 1,
                    });
                }
                (None, None) => {}
            }
        }
        assert!(open.is_none(), "{}: a fence is left open", self.file_name);
        blocks
    }

    /// The chapter without its fenced blocks.
    fn prose(&self) -> String {
        let mut prose = String::new();
        let mut is_inside_fence = false;
        for line in self.text.lines() {
            if line.trim_start().starts_with(FENCE) {
                is_inside_fence = !is_inside_fence;
            } else if !is_inside_fence {
                prose.push_str(line);
                prose.push('\n');
            }
        }
        prose
    }

    /// The texts the prose of the chapter sets in backticks.
    fn code_spans(&self) -> Vec<String> {
        let prose = self.prose();
        let mut spans = Vec::new();
        let mut rest = prose.as_str();
        while let Some(start) = rest.find('`') {
            let opened = &rest[start..];
            let delimiter_length = opened.chars().take_while(|character| *character == '`').count();
            let (delimiter, content) = opened.split_at(delimiter_length);
            let Some(end) = content.find(delimiter) else {
                break;
            };
            spans.push(content[..end].trim().to_owned());
            rest = &content[end + delimiter_length..];
        }
        spans
    }

    /// Where the links of the prose of the chapter lead.
    fn link_targets(&self) -> Vec<String> {
        let prose = self.prose();
        prose
            .split("](")
            .skip(1)
            .filter_map(|after_opening| after_opening.split_once(')'))
            .map(|(target, _)| target.to_owned())
            .collect()
    }

    /// The chapters of the guide the chapter links to, by their file names.
    fn linked_chapters(&self) -> BTreeSet<String> {
        self.link_targets()
            .iter()
            .map(|target| target.split('#').next().unwrap_or_default().to_owned())
            .collect()
    }

    /// The paths of the repository the chapter names, in backticks or as the target of a link,
    /// each without the line number after it.
    fn repository_paths(&self) -> BTreeSet<String> {
        self.code_spans()
            .into_iter()
            .chain(self.link_targets())
            .filter(|named| {
                REPOSITORY_DIRECTORIES
                    .iter()
                    .any(|directory| named.starts_with(directory))
            })
            .map(|named| named.split(':').next().unwrap_or_default().to_owned())
            .collect()
    }

    /// The names the rows of the table under the heading that names `module` open with.
    fn table_names(&self, module: &str) -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        let mut is_under_heading = false;
        for line in self.text.lines() {
            if line.starts_with('#') {
                is_under_heading = line.ends_with(module);
            } else if is_under_heading
                && let Some(name) = line.strip_prefix("| `").and_then(|row| row.split('`').next())
            {
                names.insert(name.to_owned());
            }
        }
        names
    }
}

fn workspace_file(relative_path: &str) -> String {
    let path = Path::new(WORKSPACE_ROOT).join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{} is readable: {error}", path.display()))
}

/// The names a source file exports with its `pub use` and `pub type` items.
fn exported_names(source: &str) -> BTreeSet<String> {
    source.split(';').flat_map(names_of_item).collect()
}

/// The names an item exports when it is a `pub use` or a `pub type` item; `item` is the source
/// up to the semicolon of the item.
fn names_of_item(item: &str) -> Vec<String> {
    if let Some((_, alias)) = item.rsplit_once(TYPE_ALIAS) {
        let name = alias.split(['=', '<', ' ']).next().unwrap_or_default();
        return vec![name.to_owned()];
    }
    let Some((_, reexport)) = item.rsplit_once(REEXPORT) else {
        return Vec::new();
    };
    let paths = match reexport.split_once('{') {
        Some((_, group)) => group.trim_end().trim_end_matches('}'),
        None => reexport,
    };
    paths
        .split(',')
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(|path| {
            let name = path.rsplit(" as ").next().unwrap_or(path);
            name.rsplit("::").next().unwrap_or(name).to_owned()
        })
        .collect()
}

#[test]
fn the_index_has_a_row_for_every_exported_name() {
    let index = Chapter::named(INDEX);
    for (module, source) in INDEX_TABLES {
        let exported = exported_names(&workspace_file(source));
        let indexed = index.table_names(module);

        let missing: Vec<&String> = exported.difference(&indexed).collect();

        assert!(
            missing.is_empty(),
            "the table of {module} has no row for {missing:?}, exported by {source}"
        );
    }
}

#[test]
fn every_row_of_the_index_names_an_exported_name() {
    let index = Chapter::named(INDEX);
    for (module, source) in INDEX_TABLES {
        let exported = exported_names(&workspace_file(source));
        let indexed = index.table_names(module);

        let unknown: Vec<&String> = indexed.difference(&exported).collect();

        assert!(!indexed.is_empty(), "the index has a table of {module}");
        assert!(
            unknown.is_empty(),
            "the table of {module} names {unknown:?}, which {source} does not export"
        );
    }
}

#[test]
fn every_chapter_is_attached_to_the_documentation_of_the_facade() {
    let guide_module = fs::read_to_string(GUIDE_MODULE).expect("the module of the guide is readable");

    let unattached: Vec<String> = Chapter::all()
        .into_iter()
        .map(|chapter| chapter.file_name)
        .filter(|file_name| !guide_module.contains(&format!("include_str!(\"../docs/{file_name}\")")))
        .collect();

    assert!(
        unattached.is_empty(),
        "{GUIDE_MODULE} attaches no module to {unattached:?}"
    );
}

#[test]
fn no_fence_is_tagged_ignore() {
    let ignored: Vec<String> = Chapter::all()
        .iter()
        .flat_map(|chapter| {
            chapter
                .fenced_blocks()
                .into_iter()
                .filter(|block| block.tags.iter().any(|tag| tag == IGNORED))
                .map(|block| format!("{}:{}", chapter.file_name, block.line))
                .collect::<Vec<_>>()
        })
        .collect();

    assert!(ignored.is_empty(), "a fence nothing compiles: {ignored:?}");
}

#[test]
fn every_css_fence_compiles_as_a_stylesheet_over_the_tokens_of_the_kit() {
    let tokens = compile_tokens(&SourceFile::new(KIT_TOKENS.path, KIT_TOKENS.text))
        .unwrap_or_else(|diagnostics| panic!("the tokens of the kit compile:\n{diagnostics}"));
    let mut compiled = 0;
    for chapter in Chapter::all() {
        for block in chapter.fenced_blocks() {
            if block.tags.first().map(String::as_str) != Some(STYLESHEET_LANGUAGE) {
                continue;
            }
            let stylesheet = SourceFile::new(format!("{}:{}", chapter.file_name, block.line), block.body);

            compile_stylesheets(&[stylesheet], &tokens).unwrap_or_else(|diagnostics| {
                panic!(
                    "the `css` fence at {}:{} is no stylesheet of the subset:\n{diagnostics}",
                    chapter.file_name, block.line
                )
            });
            compiled += 1;
        }
    }

    assert!(compiled > 0, "the guide shows a stylesheet");
}

#[test]
fn every_path_of_the_repository_the_guide_names_exists() {
    let missing: Vec<String> = Chapter::all()
        .iter()
        .flat_map(|chapter| {
            chapter
                .repository_paths()
                .into_iter()
                .filter(|path| !Path::new(WORKSPACE_ROOT).join(path).exists())
                .map(|path| format!("{} names {path}", chapter.file_name))
                .collect::<Vec<_>>()
        })
        .collect();

    assert!(missing.is_empty(), "paths that do not exist: {missing:#?}");
}

#[test]
fn the_overview_leads_to_every_chapter() {
    let linked = Chapter::named(OVERVIEW).linked_chapters();

    let unreachable: Vec<String> = Chapter::all()
        .into_iter()
        .map(|chapter| chapter.file_name)
        .filter(|file_name| file_name != OVERVIEW && !linked.contains(file_name))
        .collect();

    assert!(unreachable.is_empty(), "the overview links to no {unreachable:?}");
}
