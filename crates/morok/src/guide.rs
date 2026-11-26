//! The guide of the kit: how an application is built on it, one module per chapter. The chapters
//! are the files of `crates/morok/docs`, and every Rust example in them is a doctest of this
//! crate. [`overview`] is where to begin.

#[doc = include_str!("../docs/overview.md")]
pub mod overview {}

#[doc = include_str!("../docs/application.md")]
pub mod application {}

#[doc = include_str!("../docs/elements.md")]
pub mod elements {}

#[doc = include_str!("../docs/signals.md")]
pub mod signals {}

#[doc = include_str!("../docs/components.md")]
pub mod components {}

#[doc = include_str!("../docs/styles.md")]
pub mod styles {}

#[doc = include_str!("../docs/overlays.md")]
pub mod overlays {}

#[doc = include_str!("../docs/pictures.md")]
pub mod pictures {}

#[doc = include_str!("../docs/rules.md")]
pub mod rules {}

#[doc = include_str!("../docs/index.md")]
pub mod index {}
