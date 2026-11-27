//! The command that writes frames of the kit to files: what the kit itself draws of the cases
//! a list names. Chrome is not needed.

use std::fs;
use std::io;
use std::path::Path;

use morok_platform::FrameImage;

use crate::compiled_suite::CompiledSuite;
use crate::frame_list::FrameList;
use crate::frame_png;
use crate::morok_side::MorokSide;
use crate::stand_error::StandError;

/// Writes every frame of the list at `list_file` as a PNG — what the kit draws of the case
/// under the color scheme of the frame — and returns the number of frames written. A list
/// with a frame the stand cannot serve is refused before anything is drawn.
pub fn write(list_file: &Path) -> Result<usize, StandError> {
    let list = FrameList::read(list_file)?;
    let mut morok = MorokSide::new()?;
    for suite in &list.suites {
        let compiled = CompiledSuite::compile(suite)?;
        for frame in list.frames_of(suite) {
            let rendering = morok.render_case(&compiled, &frame.case);
            write_png(&frame.path, &rendering.under(frame.color_scheme).image)?;
        }
    }
    Ok(list.frames.len())
}

fn write_png(path: &Path, image: &FrameImage) -> Result<(), StandError> {
    let directory = path
        .parent()
        .expect("the file of a listed frame lies under the directory of its list");
    fs::create_dir_all(directory).map_err(|error| file_error(directory, error))?;
    fs::write(path, frame_png::encode(image)).map_err(|error| file_error(path, error))
}

fn file_error(path: &Path, error: io::Error) -> StandError {
    StandError::File {
        path: path.to_path_buf(),
        error,
    }
}
