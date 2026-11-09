use morok_core::{ElementId, ElementTree, Role};
use morok_css::{CompiledTokens, SourceFile, compile_stylesheets, compile_tokens};
use morok_style::{ColorScheme, StyleSheet, Theme};

use crate::bench_error::BenchError;
use crate::scene::Scene;

const TOKENS_PATH: &str = "design/system/tokens.css";
/// The canonical tokens of the kit: a scene is styled with what the catalog is styled with.
const TOKENS: &str = include_str!("../../../design/system/tokens.css");

/// The element tree of a scene, styled by the stylesheet of the scene over the tokens of the
/// kit. A scene builds it through the methods that add elements.
#[derive(Debug)]
pub struct SceneTree {
    tree: ElementTree,
    style_sheet: StyleSheet,
    tokens: CompiledTokens,
}

impl SceneTree {
    /// Compiles the stylesheet of the scene and lets the scene build its tree under the theme of
    /// `color_scheme`.
    pub fn of(scene: &mut dyn Scene, color_scheme: ColorScheme) -> Result<Self, BenchError> {
        let tokens = compile_tokens(&SourceFile::new(TOKENS_PATH, TOKENS)).map_err(BenchError::Rejected)?;
        let style_sheet = compile_stylesheets(&[scene.stylesheet()], &tokens).map_err(BenchError::Rejected)?;
        let mut scene_tree = Self {
            tree: ElementTree::new(style_sheet.clone(), theme_of(&tokens, color_scheme)),
            style_sheet,
            tokens,
        };
        scene.build(&mut scene_tree);
        Ok(scene_tree)
    }

    pub fn root(&self) -> ElementId {
        self.tree.root()
    }

    /// Adds a box with the given classes as the last child of `parent`.
    ///
    /// # Panics
    ///
    /// When the stylesheet of the scene names no such class.
    pub fn add(&mut self, parent: ElementId, class_names: &[&str]) -> ElementId {
        self.add_with_role(parent, Role::Element, class_names)
    }

    /// Adds a text with the given classes as the last child of `parent`.
    ///
    /// # Panics
    ///
    /// When the stylesheet of the scene names no such class.
    pub fn add_text(&mut self, parent: ElementId, class_names: &[&str], text: &str) -> ElementId {
        let element = self.add_with_role(parent, Role::Text, class_names);
        self.tree.set_text(element, text);
        element
    }

    /// Gives the given classes to an element that already exists, the root among them.
    ///
    /// # Panics
    ///
    /// When the stylesheet of the scene names no such class.
    pub fn add_classes(&mut self, element: ElementId, class_names: &[&str]) {
        for name in class_names {
            let class = self
                .style_sheet
                .class_named(name)
                .unwrap_or_else(|| panic!("the stylesheet of the scene names no class `{name}`"));
            self.tree.add_class(element, class);
        }
    }

    pub fn tree(&self) -> &ElementTree {
        &self.tree
    }

    pub fn tree_mut(&mut self) -> &mut ElementTree {
        &mut self.tree
    }

    /// Restyles the tree for the color scheme the surface switched to.
    pub fn set_color_scheme(&mut self, color_scheme: ColorScheme) {
        self.tree.set_theme(theme_of(&self.tokens, color_scheme));
    }

    fn add_with_role(&mut self, parent: ElementId, role: Role, class_names: &[&str]) -> ElementId {
        let element = self.tree.create(role);
        self.add_classes(element, class_names);
        self.tree.append_child(parent, element);
        element
    }
}

fn theme_of(tokens: &CompiledTokens, color_scheme: ColorScheme) -> Theme {
    match color_scheme {
        ColorScheme::Light => tokens.light.clone(),
        ColorScheme::Dark => tokens.dark.clone(),
    }
}
