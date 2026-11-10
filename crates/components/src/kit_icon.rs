/// An icon of the kit: one of the SVG files of the design book. An icon drawn in `currentColor`
/// takes the `color` of its element. A host registers each with its renderer once, and every
/// surface opened after that shows them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum KitIcon {
    Bolt,
    /// The mark of a checked check box.
    Check,
    Cloud,
    Copy,
    Cross,
    Disk,
    /// A chevron that points down: what opens below.
    Down,
    Eye,
    File,
    /// The one icon of more than one color.
    Flag,
    Folder,
    Gear,
    Grid,
    Home,
    Image,
    Mic,
    Plus,
    /// A chevron that points right: what opens beside, or a level deeper.
    Right,
    /// The magnifier of a search field.
    Search,
    Sidebar,
    Sort,
    Star,
    Trash,
}

impl KitIcon {
    pub const ALL: [Self; 23] = [
        Self::Bolt,
        Self::Check,
        Self::Cloud,
        Self::Copy,
        Self::Cross,
        Self::Disk,
        Self::Down,
        Self::Eye,
        Self::File,
        Self::Flag,
        Self::Folder,
        Self::Gear,
        Self::Grid,
        Self::Home,
        Self::Image,
        Self::Mic,
        Self::Plus,
        Self::Right,
        Self::Search,
        Self::Sidebar,
        Self::Sort,
        Self::Star,
        Self::Trash,
    ];

    /// The SVG of the icon, as the design book draws it.
    pub const fn svg(self) -> &'static str {
        match self {
            Self::Bolt => include_str!("../../../design/icons/bolt.svg"),
            Self::Check => include_str!("../../../design/icons/check.svg"),
            Self::Cloud => include_str!("../../../design/icons/cloud.svg"),
            Self::Copy => include_str!("../../../design/icons/copy.svg"),
            Self::Cross => include_str!("../../../design/icons/x.svg"),
            Self::Disk => include_str!("../../../design/icons/disk.svg"),
            Self::Down => include_str!("../../../design/icons/down.svg"),
            Self::Eye => include_str!("../../../design/icons/eye.svg"),
            Self::File => include_str!("../../../design/icons/file.svg"),
            Self::Flag => include_str!("../../../design/icons/flag.svg"),
            Self::Folder => include_str!("../../../design/icons/folder.svg"),
            Self::Gear => include_str!("../../../design/icons/gear.svg"),
            Self::Grid => include_str!("../../../design/icons/grid.svg"),
            Self::Home => include_str!("../../../design/icons/home.svg"),
            Self::Image => include_str!("../../../design/icons/image.svg"),
            Self::Mic => include_str!("../../../design/icons/mic.svg"),
            Self::Plus => include_str!("../../../design/icons/plus.svg"),
            Self::Right => include_str!("../../../design/icons/right.svg"),
            Self::Search => include_str!("../../../design/icons/search.svg"),
            Self::Sidebar => include_str!("../../../design/icons/sidebar.svg"),
            Self::Sort => include_str!("../../../design/icons/sort.svg"),
            Self::Star => include_str!("../../../design/icons/star.svg"),
            Self::Trash => include_str!("../../../design/icons/trash.svg"),
        }
    }

    /// The place of the icon in [`Self::ALL`].
    pub(crate) const fn index(self) -> usize {
        self as usize
    }
}
