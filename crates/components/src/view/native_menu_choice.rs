use morok_reactive::Runtime;

use crate::view::surfaces::Surfaces;

/// What a component does once the user chose a line of the menu of the system it asked for:
/// it is handed the `id` of the line.
pub type NativeMenuChoice = Box<dyn FnMut(&mut Runtime<Surfaces>, usize)>;
