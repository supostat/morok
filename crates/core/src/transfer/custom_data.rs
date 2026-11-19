/// Data of a type of the application's own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomData {
    /// The uniform type identifier of the type, such as `com.example.notes.note`.
    pub type_name: String,
    pub bytes: Vec<u8>,
}
