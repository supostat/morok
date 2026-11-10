use morok_css::{CompiledTokens, SourceFile, compile_stylesheets, compile_tokens};
use morok_style::{ColorScheme, StyleSheet};

use crate::case::Case;
use crate::chrome_build::ChromeBuild;
use crate::harness_page;
use crate::manifest::Manifest;
use crate::marker_classes;
use crate::markup;
use crate::moment::Moment;
use crate::scene::Scene;
use crate::stand_error::StandError;
use crate::suite::Suite;

const TOKENS_PATH: &str = "design/system/tokens.css";
/// The canonical tokens of the kit: the compiler of the kit reads them and Chrome gets them as
/// they are.
const TOKENS: &str = include_str!("../../../design/system/tokens.css");
const STYLE_END_TAG: &str = "</style";

/// A suite whose stylesheets the compiler of the kit has accepted: both sides of the stand
/// render its cases from here.
#[derive(Debug)]
pub struct CompiledSuite<'a> {
    suite: &'a Suite,
    tokens: CompiledTokens,
    style_sheet: StyleSheet,
}

impl<'a> CompiledSuite<'a> {
    pub fn compile(suite: &'a Suite) -> Result<Self, StandError> {
        if let Some(unembeddable) = suite
            .stylesheets
            .iter()
            .find(|stylesheet| closes_style_element(stylesheet))
        {
            return Err(StandError::Unembeddable {
                path: unembeddable.path().to_path_buf(),
            });
        }
        let tokens = compile_tokens(&SourceFile::new(TOKENS_PATH, TOKENS)).map_err(StandError::Rejected)?;
        let style_sheet = compile_stylesheets(&suite.stylesheets, &tokens).map_err(StandError::Rejected)?;
        Ok(Self {
            suite,
            tokens,
            style_sheet,
        })
    }

    pub fn suite(&self) -> &'a Suite {
        self.suite
    }

    /// The tree of the case under the theme of the color scheme.
    pub(crate) fn scene(&self, case: &Case, color_scheme: ColorScheme) -> Scene {
        let theme = match color_scheme {
            ColorScheme::Light => self.tokens.light.clone(),
            ColorScheme::Dark => self.tokens.dark.clone(),
        };
        let mut scene = Scene::new(self.style_sheet.clone(), theme);
        (case.build)(&mut scene);
        scene
    }

    /// The page Chrome renders the case from. The tree of a case is the same under both
    /// themes, and so is its page: Chrome picks the tokens of a scheme itself.
    pub(crate) fn page(&self, case: &Case) -> String {
        let stylesheets: Vec<String> = self
            .suite
            .stylesheets
            .iter()
            .map(|stylesheet| marker_classes::in_place_of_states(stylesheet.text()))
            .collect();
        harness_page::document(TOKENS, &stylesheets, &self.markup(case))
    }

    /// What an etalon of the case rendered now by `chrome_build` would be made of.
    pub fn manifest(&self, case: &Case, chrome_build: &ChromeBuild) -> Manifest {
        let stylesheets: Vec<&str> = self.suite.stylesheets.iter().map(SourceFile::text).collect();
        Manifest::of(
            &stylesheets,
            &harness_page::document("", &[], &self.markup(case)),
            TOKENS,
            &chrome_build.version,
        )
    }

    /// The markup of the tree before any change, with the motion of the case for the page to
    /// play when it has one.
    fn markup(&self, case: &Case) -> String {
        let mut scene = self.scene(case, ColorScheme::Light);
        let mut markup = markup::of(&scene);
        if let Moment::InMotion { change, after } = case.moment {
            change(&mut scene);
            markup.push_str(&markup::motion_script(&scene, after));
        }
        markup
    }
}

/// Whether the text of the stylesheet would end the `style` element that embeds it.
fn closes_style_element(stylesheet: &SourceFile) -> bool {
    stylesheet.text().to_ascii_lowercase().contains(STYLE_END_TAG)
}
