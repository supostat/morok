use std::fmt::Write;

use crate::folder_usage::FolderUsage;
use crate::picked_folder::PickedFolder;
use crate::sunburst::Sunburst;
use crate::tokens::TOKENS;
use crate::treemap::Treemap;

/// The gigabytes of a terabyte, as a disk is sold.
const GIGABYTES_PER_TERABYTE: f32 = 1000.0;

/// The folders of Macintosh HD: 612 GB of a terabyte.
const MACINTOSH_FOLDERS: [FolderUsage; 6] = [
    FolderUsage {
        name: "Developer",
        gigabytes: 60.0,
        children: &[38.0, 22.0],
        color: TOKENS.color_chart_1,
    },
    FolderUsage {
        name: "Caches",
        gigabytes: 42.0,
        children: &[30.0, 8.0, 4.0],
        color: TOKENS.color_chart_2,
    },
    FolderUsage {
        name: "Photos",
        gigabytes: 180.0,
        children: &[90.0, 50.0, 40.0],
        color: TOKENS.color_chart_3,
    },
    FolderUsage {
        name: "Projects",
        gigabytes: 120.0,
        children: &[70.0, 30.0, 20.0],
        color: TOKENS.color_chart_4,
    },
    FolderUsage {
        name: "Music",
        gigabytes: 95.0,
        children: &[60.0, 35.0],
        color: TOKENS.color_chart_5,
    },
    FolderUsage {
        name: "System",
        gigabytes: 115.0,
        children: &[40.0, 40.0, 35.0],
        color: TOKENS.color_chart_6,
    },
];

/// The folders of Archive: 2.9 TB of four terabytes.
const ARCHIVE_FOLDERS: [FolderUsage; 6] = [
    FolderUsage {
        name: "Footage",
        gigabytes: 1200.0,
        children: &[700.0, 300.0, 200.0],
        color: TOKENS.color_chart_1,
    },
    FolderUsage {
        name: "Backups",
        gigabytes: 800.0,
        children: &[500.0, 300.0],
        color: TOKENS.color_chart_2,
    },
    FolderUsage {
        name: "Photos",
        gigabytes: 420.0,
        children: &[250.0, 120.0, 50.0],
        color: TOKENS.color_chart_3,
    },
    FolderUsage {
        name: "Old projects",
        gigabytes: 260.0,
        children: &[160.0, 100.0],
        color: TOKENS.color_chart_4,
    },
    FolderUsage {
        name: "Installers",
        gigabytes: 140.0,
        children: &[90.0, 50.0],
        color: TOKENS.color_chart_5,
    },
    FolderUsage {
        name: "Documents",
        gigabytes: 80.0,
        children: &[50.0, 30.0],
        color: TOKENS.color_chart_6,
    },
];

/// A volume the disk map analyses: its folders, what they use of it, and the folder its
/// analysis picks out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Volume {
    Macintosh,
    Archive,
}

impl Volume {
    pub const ALL: [Self; 2] = [Self::Macintosh, Self::Archive];

    pub fn name(self) -> &'static str {
        match self {
            Self::Macintosh => "Macintosh HD",
            Self::Archive => "Archive",
        }
    }

    pub fn capacity(self) -> &'static str {
        match self {
            Self::Macintosh => "1 TB",
            Self::Archive => "4 TB",
        }
    }

    /// The folders of the volume, in the order the charts draw them.
    pub fn folders(self) -> &'static [FolderUsage] {
        match self {
            Self::Macintosh => &MACINTOSH_FOLDERS,
            Self::Archive => &ARCHIVE_FOLDERS,
        }
    }

    /// What the folders of the volume use of it, as the disk map writes it: in gigabytes, or
    /// in terabytes from a thousand gigabytes on.
    pub fn used(self) -> String {
        let gigabytes: f32 = self.folders().iter().map(|folder| folder.gigabytes).sum();
        if gigabytes < GIGABYTES_PER_TERABYTE {
            format!("{gigabytes} GB")
        } else {
            format!("{:.1} TB", gigabytes / GIGABYTES_PER_TERABYTE)
        }
    }

    pub fn sunburst(self) -> Sunburst {
        Sunburst::new(self.folders())
    }

    pub fn treemap(self) -> Treemap {
        Treemap::new(self.folders())
    }

    pub fn picked_folder(self) -> PickedFolder {
        match self {
            Self::Macintosh => PickedFolder {
                path: "~/Library/Developer",
                size: "60.1 GB · 214,038 files",
                share: 0.38,
                note: "Local model: Xcode build caches and simulators, rebuilt automatically.",
            },
            Self::Archive => PickedFolder {
                path: "/Volumes/Archive/Footage",
                size: "1.2 TB · 3,412 files",
                share: 0.41,
                note: "Local model: camera originals, already imported into Photos.",
            },
        }
    }

    /// What the charts show, told in words: the used space, then every folder with its size.
    pub fn description(self) -> String {
        let mut description = format!("{}, {} used of {}:", self.name(), self.used(), self.capacity());
        for (position, folder) in self.folders().iter().enumerate() {
            let separator = if position == 0 { " " } else { ", " };
            write!(description, "{separator}{} {} GB", folder.name, folder.gigabytes)
                .expect("a string takes what is written to it");
        }
        description
    }
}
