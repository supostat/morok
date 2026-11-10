use std::borrow::Cow;

use morok_style::{Declaration, Keyframe, Keyframes, KeyframesRule};

use crate::keyframes_table::KeyframesTable;

/// The `@keyframes` rules read so far, by ID.
#[derive(Debug)]
pub(crate) struct KeyframesRules {
    drafts: Vec<Option<KeyframesDraft>>,
}

#[derive(Debug)]
struct KeyframesDraft {
    name: String,
    /// In source order, before keyframes of the same offset are merged.
    keyframes: Vec<(f32, Vec<Declaration>)>,
}

impl KeyframesRules {
    pub(crate) fn new(table: &KeyframesTable) -> Self {
        Self {
            drafts: table.names().iter().map(|_| None).collect(),
        }
    }

    /// Whether the rule of the ID has been read already.
    pub(crate) fn is_declared(&self, keyframes: Keyframes) -> bool {
        self.drafts[keyframes.index()].is_some()
    }

    pub(crate) fn declare(&mut self, keyframes: Keyframes, name: &str) {
        self.drafts[keyframes.index()] = Some(KeyframesDraft {
            name: name.to_owned(),
            keyframes: Vec::new(),
        });
    }

    pub(crate) fn add_keyframe(&mut self, keyframes: Keyframes, offset: f32, declarations: Vec<Declaration>) {
        self.drafts[keyframes.index()]
            .as_mut()
            .expect("a keyframe belongs to a declared rule")
            .keyframes
            .push((offset, declarations));
    }

    /// The rules in ID order, each with its keyframes ordered by offset and the keyframes of one
    /// offset merged in source order.
    ///
    /// # Panics
    ///
    /// When a rule the table names was not read: every rule of the table is read before.
    pub(crate) fn into_rules(self) -> Vec<KeyframesRule> {
        self.drafts
            .into_iter()
            .map(|draft| {
                let mut draft = draft.expect("every named rule is read");
                draft.keyframes.sort_by(|(left, _), (right, _)| left.total_cmp(right));
                let mut keyframes: Vec<Keyframe> = Vec::new();
                for (offset, declarations) in draft.keyframes {
                    match keyframes.last_mut() {
                        Some(last) if last.offset.to_bits() == offset.to_bits() => {
                            last.declarations.to_mut().extend(declarations);
                        }
                        _ => keyframes.push(Keyframe {
                            offset,
                            declarations: Cow::Owned(declarations),
                        }),
                    }
                }
                KeyframesRule {
                    name: Cow::Owned(draft.name),
                    keyframes: Cow::Owned(keyframes),
                }
            })
            .collect()
    }
}
