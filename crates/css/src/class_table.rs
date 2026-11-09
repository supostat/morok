use std::collections::HashMap;

use morok_style::Class;

/// Class names in the order they first appear; the order gives each class its ID.
#[derive(Debug, Default)]
pub(crate) struct ClassTable {
    names: Vec<String>,
    classes: HashMap<String, Class>,
}

impl ClassTable {
    pub(crate) fn class(&mut self, name: &str) -> Class {
        if let Some(class) = self.classes.get(name) {
            return *class;
        }
        let index = u32::try_from(self.names.len()).expect("a stylesheet holds fewer than 2^32 classes");
        let class = Class::new(index);
        self.names.push(name.to_owned());
        self.classes.insert(name.to_owned(), class);
        class
    }

    pub(crate) fn into_names(self) -> Vec<String> {
        self.names
    }
}
