mod support;

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use morok_core::{Clipboard, CustomData, Payload, PayloadType};
use morok_platform::PasteboardClipboard;
use objc2::rc::Retained;
use objc2::{MainThreadMarker, msg_send};
use objc2_app_kit::NSPasteboard;
use support::Expectations;

/// Two types of an application's own.
const NOTE_TYPE: &str = "com.example.notes.note";
const FOLDER_TYPE: &str = "com.example.notes.folder";

/// A pasteboard of the system under a name of its own, written through one clipboard and read
/// through another that reaches it by its name.
struct Stage {
    pasteboard: Retained<NSPasteboard>,
    writer: PasteboardClipboard,
    reader: PasteboardClipboard,
    /// A folder of the test's own, with a file in it.
    folder: PathBuf,
    expectations: Expectations,
}

impl Stage {
    fn open() -> Self {
        let pasteboard = NSPasteboard::pasteboardWithUniqueName();
        let by_name = NSPasteboard::pasteboardWithName(&pasteboard.name());
        let folder = std::env::temp_dir().join(format!("morok pasteboard types {}", std::process::id()));
        fs::create_dir_all(&folder).expect("the test makes a folder of its own");
        fs::write(folder.join("report.txt"), "a report").expect("the test writes a file of its own");
        Self {
            writer: PasteboardClipboard::of(pasteboard.clone()),
            reader: PasteboardClipboard::of(by_name),
            pasteboard,
            folder,
            expectations: Expectations::default(),
        }
    }

    fn every_type() -> [PayloadType; 5] {
        [
            PayloadType::Text,
            PayloadType::Files,
            PayloadType::Urls,
            PayloadType::Custom(String::from(NOTE_TYPE)),
            PayloadType::Custom(String::from(FOLDER_TYPE)),
        ]
    }

    /// Writes the payload and expects to read it back whole, asking for every type.
    fn round_trips(&mut self, payload: &Payload, expectation: &str) {
        self.writer.write(payload);
        let read = self.reader.read(&Self::every_type());
        self.expectations
            .expect(read == *payload, expectation, &format!("{read:?}"));
    }

    fn files(&self) -> Vec<PathBuf> {
        vec![
            self.folder.join("report.txt"),
            self.folder.clone(),
            self.folder
                .join("not written yet/\u{43e}\u{442}\u{447}\u{451}\u{442} 2026.pdf"),
        ]
    }

    fn text_round_trips(&mut self) {
        let text = Payload::of_text("na\u{ef}ve caf\u{e9}\n\u{2615} two lines");
        self.round_trips(&text, "a text is read back as it was written");
    }

    fn files_round_trip(&mut self) {
        let files = Payload {
            files: self.files(),
            ..Payload::default()
        };
        self.round_trips(
            &files,
            "a file, a folder and a path nothing lies at yet are read back as files, in their order, and as nothing else",
        );
    }

    fn urls_round_trip(&mut self) {
        let urls = Payload {
            urls: vec![
                String::from("https://example.com/list?page=2#top"),
                String::from("mailto:someone@example.com"),
            ],
            ..Payload::default()
        };
        self.round_trips(
            &urls,
            "addresses that name no file are read back as addresses, in their order, and as nothing else",
        );
    }

    fn a_custom_type_round_trips(&mut self) {
        let note = Payload {
            custom: vec![CustomData {
                type_name: String::from(NOTE_TYPE),
                bytes: vec![0, 255, 7, 0, 42],
            }],
            ..Payload::default()
        };
        self.round_trips(
            &note,
            "the bytes of a type of the application's own are read back under its name",
        );
    }

    fn every_type_round_trips_at_once(&mut self) {
        let everything = Payload {
            text: Some(String::from("report.txt and a note")),
            files: self.files(),
            urls: vec![String::from("https://example.com/")],
            custom: vec![
                CustomData {
                    type_name: String::from(NOTE_TYPE),
                    bytes: vec![1, 2, 3],
                },
                CustomData {
                    type_name: String::from(FOLDER_TYPE),
                    bytes: Vec::new(),
                },
            ],
        };
        self.round_trips(&everything, "a payload of every type is read back whole");

        let files_alone = self.reader.read(&[PayloadType::Files]);
        self.expectations.expect(
            files_alone
                == Payload {
                    files: self.files(),
                    ..Payload::default()
                },
            "only the types asked for are read",
            &format!("{files_alone:?}"),
        );
        let nothing_asked = self.reader.read(&[]);
        self.expectations.expect(
            nothing_asked == Payload::default(),
            "asking for no type reads nothing",
            &format!("{nothing_asked:?}"),
        );
    }

    fn a_write_replaces_what_the_pasteboard_held(&mut self) {
        self.writer.write(&Payload {
            files: self.files(),
            ..Payload::default()
        });
        self.writer.write(&Payload::of_text("only a text"));
        let read = self.reader.read(&Self::every_type());
        self.expectations.expect(
            read == Payload::of_text("only a text"),
            "what was written last is all the pasteboard holds",
            &format!("{read:?}"),
        );

        self.writer.write(&Payload::default());
        let read = self.reader.read(&Self::every_type());
        self.expectations.expect(
            read == Payload::default(),
            "an empty payload leaves the pasteboard empty",
            &format!("{read:?}"),
        );
    }

    fn close(self) -> ExitCode {
        fs::remove_dir_all(&self.folder).expect("the test removes its folder");
        // SAFETY: a pasteboard with a unique name is released once nothing reads it any more.
        let (): () = unsafe { msg_send![&*self.pasteboard, releaseGlobally] };
        self.expectations.report("pasteboard types")
    }
}

fn main() -> ExitCode {
    MainThreadMarker::new().expect("the test runs on the main thread");
    let mut stage = Stage::open();

    stage.text_round_trips();
    stage.files_round_trip();
    stage.urls_round_trip();
    stage.a_custom_type_round_trips();
    stage.every_type_round_trips_at_once();
    stage.a_write_replaces_what_the_pasteboard_held();

    stage.close()
}
