use morok::components::view::Mount;
use morok::components::{KitIcon, ScrollView};
use morok::core::ElementId;

use crate::disk_map::DiskMap;
use crate::file_manager::FileManager;
use crate::gallery::Gallery;
use crate::pages::{
    buttons_page, collections_page, controls_page, icons_page, images_page, overlays_page, pickers_page,
    scaffolding_page, table_page, text_fields_page,
};

/// A page of the catalog: a family of the components of the kit, or a screen built of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Buttons,
    Icons,
    Images,
    TextFields,
    Controls,
    Collections,
    Table,
    Overlays,
    Pickers,
    Scaffolding,
    FileManager,
    DiskMap,
}

impl Page {
    /// The pages of the component families, in the order of the sidebar.
    pub const FAMILIES: [Self; 10] = [
        Self::Buttons,
        Self::Icons,
        Self::Images,
        Self::TextFields,
        Self::Controls,
        Self::Collections,
        Self::Table,
        Self::Overlays,
        Self::Pickers,
        Self::Scaffolding,
    ];
    /// The screens built of the components.
    pub const SCREENS: [Self; 2] = [Self::FileManager, Self::DiskMap];

    pub fn title(self) -> &'static str {
        match self {
            Self::Buttons => "Buttons",
            Self::Icons => "Icons",
            Self::Images => "Images",
            Self::TextFields => "Text fields",
            Self::Controls => "Controls",
            Self::Collections => "Collections",
            Self::Table => "Table",
            Self::Overlays => "Overlays",
            Self::Pickers => "Select and combobox",
            Self::Scaffolding => "Window scaffolding",
            Self::FileManager => "File manager",
            Self::DiskMap => "Disk map",
        }
    }

    /// The components the page shows.
    pub fn components(self) -> &'static str {
        match self {
            Self::Buttons => "Button · IconButton",
            Self::Icons => "the icons of the kit",
            Self::Images => "Image",
            Self::TextFields => "TextField · SearchField · TextArea",
            Self::Controls => "Checkbox · RadioGroup · Switch · Slider · SegmentedControl · ProgressBar · Spinner",
            Self::Collections => "List · Tree · Tabs",
            Self::Table => "Table",
            Self::Overlays => "Popover · Tooltip · Menu · ContextMenu · Dialog · Sheet · Toast",
            Self::Pickers => "Select · Combobox",
            Self::Scaffolding => "Toolbar · StatusBar · Sidebar · SplitView",
            Self::FileManager => "two tables in a split view",
            Self::DiskMap => "a sunburst on a canvas",
        }
    }

    pub fn icon(self) -> KitIcon {
        match self {
            Self::Buttons => KitIcon::Bolt,
            Self::Icons => KitIcon::Star,
            Self::Images => KitIcon::Image,
            Self::TextFields => KitIcon::Search,
            Self::Controls => KitIcon::Gear,
            Self::Collections => KitIcon::Grid,
            Self::Table => KitIcon::Sort,
            Self::Overlays => KitIcon::Copy,
            Self::Pickers => KitIcon::Down,
            Self::Scaffolding => KitIcon::Sidebar,
            Self::FileManager => KitIcon::Folder,
            Self::DiskMap => KitIcon::Disk,
        }
    }

    /// Builds the page, which shows the pictures of `gallery`. A page of a family taller than
    /// the room it has scrolls inside it; the table and the screens fill that room.
    pub fn mount(self, mount: &mut Mount<'_>, gallery: &Gallery) -> ElementId {
        let scrolled = |build: fn(&mut Mount<'_>) -> ElementId, mount: &mut Mount<'_>| {
            ScrollView::new(build).label(self.title()).mount(mount)
        };
        match self {
            Self::Buttons => scrolled(buttons_page::mount, mount),
            Self::Icons => scrolled(icons_page::mount, mount),
            Self::Images => {
                let gallery = gallery.clone();
                ScrollView::new(move |mount| images_page::mount(mount, &gallery))
                    .label(self.title())
                    .mount(mount)
            }
            Self::TextFields => scrolled(text_fields_page::mount, mount),
            Self::Controls => scrolled(controls_page::mount, mount),
            Self::Collections => scrolled(collections_page::mount, mount),
            Self::Table => table_page::mount(mount),
            Self::Overlays => overlays_page::mount(mount),
            Self::Pickers => scrolled(pickers_page::mount, mount),
            Self::Scaffolding => scrolled(scaffolding_page::mount, mount),
            Self::FileManager => FileManager::mount(mount),
            Self::DiskMap => DiskMap::mount(mount),
        }
    }
}
