//! `underrated` — an independent web browser engine, built stage by stage.
//!
//! This crate is the deterministic, platform-independent **core**: bytes →
//! encoding → HTML → DOM, plus CSS and geometry primitives. Each stage is a
//! module under `src/<module>/`. Binaries and integration tests consume the
//! engine through this library root.
//!
//! See `docs/ARCHITECTURE.md` (meta repo) for the module boundaries.

pub mod ascii;
pub mod css;
pub mod dom;
pub mod encoding;
pub mod geom;
pub mod html;
pub mod infra;
pub mod loader;
pub mod selector;
pub mod style;
pub mod url;
