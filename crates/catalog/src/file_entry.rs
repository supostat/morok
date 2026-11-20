/// A line of a pane of the file manager: a folder or a file of the place the pane shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FileEntry {
    pub(crate) name: &'static str,
    /// What the size column shows: none for a folder, whose size the pane does not count.
    pub(crate) size: Option<&'static str>,
    pub(crate) modified: &'static str,
}

impl FileEntry {
    /// The entry that leads to the folder the place lies in.
    pub(crate) const WAY_UP: Self = Self {
        name: "..",
        size: None,
        modified: "",
    };
    pub(crate) const fn folder(name: &'static str, modified: &'static str) -> Self {
        Self {
            name,
            size: None,
            modified,
        }
    }

    pub(crate) const fn file(name: &'static str, size: &'static str, modified: &'static str) -> Self {
        Self {
            name,
            size: Some(size),
            modified,
        }
    }

    pub(crate) fn is_folder(&self) -> bool {
        self.size.is_none()
    }

    pub(crate) fn is_way_up(&self) -> bool {
        *self == Self::WAY_UP
    }
}
