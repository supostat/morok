use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];
/// The signature, then the length and the name of the first chunk, then the two numbers.
const HEADER_LENGTH: usize = 24;
const WIDTH_OFFSET: usize = 16;
const HEIGHT_OFFSET: usize = 20;
const NUMBER_LENGTH: usize = 4;

/// The size a PNG file declares in its header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PngSize {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl PngSize {
    /// The size of the PNG at `path`; none when the file does not start as a PNG does. The
    /// header is all that is read: nothing of the picture is decoded.
    pub(crate) fn of(path: &Path) -> io::Result<Option<Self>> {
        let mut header = [0; HEADER_LENGTH];
        match File::open(path)?.read_exact(&mut header) {
            Ok(()) if header.starts_with(&SIGNATURE) => Ok(Some(Self {
                width: number_at(&header, WIDTH_OFFSET),
                height: number_at(&header, HEIGHT_OFFSET),
            })),
            Ok(()) => Ok(None),
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(error) => Err(error),
        }
    }
}

fn number_at(header: &[u8; HEADER_LENGTH], offset: usize) -> u32 {
    let mut number = [0; NUMBER_LENGTH];
    number.copy_from_slice(&header[offset..offset + NUMBER_LENGTH]);
    u32::from_be_bytes(number)
}
