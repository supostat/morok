use std::fmt::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::{self, Path, PathBuf};

use morok_core::{CustomData, Payload, PayloadType};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{ClassType, Message};
use objc2_app_kit::{
    NSPasteboard, NSPasteboardItem, NSPasteboardType, NSPasteboardTypeFileURL, NSPasteboardTypeString,
    NSPasteboardTypeURL, NSPasteboardWriting,
};
use objc2_foundation::{NSArray, NSData, NSString, NSURL};

/// What a pasteboard holds in the given types. A type it holds nothing in is left out of the
/// payload.
pub(crate) fn read(pasteboard: &NSPasteboard, types: &[PayloadType]) -> Payload {
    let mut payload = Payload::default();
    for payload_type in types {
        match payload_type {
            PayloadType::Text => {
                payload.text = pasteboard.stringForType(text_type()).map(|text| text.to_string());
            }
            PayloadType::Files => {
                payload.files = addresses(pasteboard)
                    .iter()
                    .filter(|address| address.isFileURL())
                    .filter_map(|file| file.filePathURL()?.path())
                    .map(|path| PathBuf::from(path.to_string()))
                    .collect();
            }
            PayloadType::Urls => {
                payload.urls = addresses(pasteboard)
                    .iter()
                    .filter(|address| !address.isFileURL())
                    .filter_map(|address| address.absoluteString())
                    .map(|address| address.to_string())
                    .collect();
            }
            PayloadType::Custom(type_name) => {
                let held = pasteboard.dataForType(&NSString::from_str(type_name));
                payload.custom.extend(held.map(|data| CustomData {
                    type_name: type_name.clone(),
                    bytes: data.to_vec(),
                }));
            }
        }
    }
    payload
}

/// Puts the payload on the pasteboard in place of what it held.
pub(crate) fn write(pasteboard: &NSPasteboard, payload: &Payload) {
    pasteboard.clearContents();
    let items = items_of(payload);
    let writers: Vec<&ProtocolObject<dyn NSPasteboardWriting>> =
        items.iter().map(|item| ProtocolObject::from_ref(&**item)).collect();
    pasteboard.writeObjects(&NSArray::from_slice(&writers));
}

/// The items a pasteboard holds the payload in: one to a file and one to an address, in that
/// order. The text and the data of the application's own types go with the first of them, so
/// that whoever takes the files is offered nothing but files; a payload of neither file nor
/// address has one item for them, and an empty payload has none.
pub(crate) fn items_of(payload: &Payload) -> Vec<Retained<NSPasteboardItem>> {
    let files = payload
        .files
        .iter()
        .filter_map(|file| file_address(file))
        .map(|address| item_holding(&address, file_type()));
    let urls = payload
        .urls
        .iter()
        .map(|url| item_holding(&NSString::from_str(url), url_type()));
    let mut items: Vec<Retained<NSPasteboardItem>> = files.chain(urls).collect();
    let holds_text_or_data = payload.text.is_some() || !payload.custom.is_empty();
    if items.is_empty() && holds_text_or_data {
        items.push(NSPasteboardItem::new());
    }
    if let Some(first) = items.first() {
        if let Some(text) = &payload.text {
            first.setString_forType(&NSString::from_str(text), text_type());
        }
        for data in &payload.custom {
            first.setData_forType(&NSData::with_bytes(&data.bytes), &NSString::from_str(&data.type_name));
        }
    }
    items
}

/// The types of a pasteboard the payload types are held in: what a view asks the system to
/// offer it the drags of.
pub(crate) fn pasteboard_types(types: &[PayloadType]) -> Retained<NSArray<NSPasteboardType>> {
    let names: Vec<Retained<NSPasteboardType>> = types
        .iter()
        .map(|payload_type| match payload_type {
            PayloadType::Text => text_type().retain(),
            PayloadType::Files => file_type().retain(),
            PayloadType::Urls => url_type().retain(),
            PayloadType::Custom(type_name) => NSString::from_str(type_name),
        })
        .collect();
    NSArray::from_retained_slice(&names)
}

/// The address of a file as a pasteboard holds it, with the bytes of its path as they are:
/// an address the system makes of a path spells the name of the file in the decomposed form
/// of its file system, and the path read back would no longer equal the one written. A path
/// that is not absolute is taken from the working directory; none for an empty path.
fn file_address(file: &Path) -> Option<Retained<NSString>> {
    let absolute = path::absolute(file).ok()?;
    let mut address = String::from("file://");
    for byte in absolute.as_os_str().as_bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/".contains(byte) {
            address.push(char::from(*byte));
        } else {
            write!(address, "%{byte:02X}").expect("a string takes what is written to it");
        }
    }
    Some(NSString::from_str(&address))
}

fn item_holding(string: &NSString, pasteboard_type: &NSPasteboardType) -> Retained<NSPasteboardItem> {
    let item = NSPasteboardItem::new();
    item.setString_forType(string, pasteboard_type);
    item
}

/// Every address the pasteboard holds, of a file or not, legacy types of the system included.
fn addresses(pasteboard: &NSPasteboard) -> Vec<Retained<NSURL>> {
    let classes = NSArray::from_slice(&[NSURL::class()]);
    // SAFETY: the class reads itself from a pasteboard, and no options are passed.
    let read = unsafe { pasteboard.readObjectsForClasses_options(&classes, None) };
    read.map_or_else(Vec::new, |objects| {
        objects
            .to_vec()
            .into_iter()
            .filter_map(|object| object.downcast::<NSURL>().ok())
            .collect()
    })
}

fn text_type() -> &'static NSPasteboardType {
    // SAFETY: the type is a constant of AppKit.
    unsafe { NSPasteboardTypeString }
}

fn file_type() -> &'static NSPasteboardType {
    // SAFETY: the type is a constant of AppKit.
    unsafe { NSPasteboardTypeFileURL }
}

fn url_type() -> &'static NSPasteboardType {
    // SAFETY: the type is a constant of AppKit.
    unsafe { NSPasteboardTypeURL }
}
