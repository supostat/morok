use morok::components::KitIcon;

use crate::file_entry::FileEntry;

/// A place a pane of the file manager shows: a folder or a volume and what lies in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FilePlace {
    pub(crate) path: &'static str,
    pub(crate) icon: KitIcon,
    /// What the header of the pane says of the place besides its path.
    pub(crate) summary: &'static str,
    /// The way up first, then the folders and the files.
    pub(crate) entries: &'static [FileEntry],
}
