use std::borrow::Cow;

use morok_style::{ClassStyle, Declaration, KeyframesRule, Selector, StyleRule, StyleSheet};

/// A rule as read from a stylesheet, before the cascade orders it.
#[derive(Debug)]
pub(crate) struct SourceRule {
    pub(crate) selector: Selector,
    /// The number of classes and pseudo-classes the selector names.
    pub(crate) specificity: u32,
    pub(crate) declarations: Vec<Declaration>,
}

/// Orders rules as the CSS cascade applies them — by specificity, then by source position — and
/// keys each rule by the first class of its subject.
pub(crate) fn style_sheet(
    mut rules: Vec<SourceRule>,
    class_names: Vec<String>,
    keyframes: Vec<KeyframesRule>,
) -> StyleSheet {
    rules.sort_by_key(|rule| rule.specificity);
    let mut keyed_rules = vec![Vec::new(); class_names.len()];
    for (index, rule) in rules.iter().enumerate() {
        let key = rule.selector.subject.classes[0];
        let index = u32::try_from(index).expect("a stylesheet holds fewer than 2^32 rules");
        keyed_rules[key.index()].push(index);
    }
    StyleSheet {
        rules: rules
            .into_iter()
            .map(|rule| StyleRule {
                selector: rule.selector,
                declarations: Cow::Owned(rule.declarations),
            })
            .collect(),
        classes: class_names
            .into_iter()
            .zip(keyed_rules)
            .map(|(name, rules)| ClassStyle {
                name: Cow::Owned(name),
                rules: Cow::Owned(rules),
            })
            .collect(),
        keyframes: Cow::Owned(keyframes),
    }
}
