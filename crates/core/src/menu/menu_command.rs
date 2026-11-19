/// The line of the menu bar a shortcut stands on.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MenuCommand {
    /// The title of the menu the line is in: lines of one title stand in one menu, in the
    /// order their shortcuts were registered.
    pub menu: String,
    /// The title of the line.
    pub title: String,
}
