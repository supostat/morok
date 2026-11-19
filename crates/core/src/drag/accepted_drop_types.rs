use crate::transfer::payload_type::PayloadType;

/// The payload types the drop targets of a tree take between them: what the host of the tree
/// asks the system to offer its surface.
#[derive(Debug, Default)]
pub(crate) struct AcceptedDropTypes {
    /// Every type with the number of drop targets that take it.
    takers: Vec<(PayloadType, usize)>,
    /// How many times the set of types changed.
    revision: u64,
}

impl AcceptedDropTypes {
    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn types(&self) -> impl Iterator<Item = &PayloadType> {
        self.takers.iter().map(|(payload_type, _)| payload_type)
    }

    /// A drop target began to take the types.
    pub(crate) fn add(&mut self, types: &[PayloadType]) {
        for added in types {
            if let Some((_, count)) = self.takers.iter_mut().find(|(held, _)| held == added) {
                *count += 1;
            } else {
                self.takers.push((added.clone(), 1));
                self.revision += 1;
            }
        }
    }

    /// A drop target takes the types no longer.
    pub(crate) fn remove(&mut self, types: &[PayloadType]) {
        for removed in types {
            let position = self
                .takers
                .iter()
                .position(|(held, _)| held == removed)
                .expect("a type a drop target took is counted");
            self.takers[position].1 -= 1;
            if self.takers[position].1 == 0 {
                self.takers.swap_remove(position);
                self.revision += 1;
            }
        }
    }
}
