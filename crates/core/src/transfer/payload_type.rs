/// A type a clipboard holds its content in and a drag carries it in.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PayloadType {
    /// Plain text.
    Text,
    /// Files and folders, by their paths.
    Files,
    /// Addresses that name no file.
    Urls,
    /// Data of a type of the application's own, by the uniform type identifier of the type,
    /// such as `com.example.notes.note`.
    Custom(String),
}
