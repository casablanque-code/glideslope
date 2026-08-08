// The app/app.rs split (rather than app/mod.rs holding App directly)
// matches the module layout in ROADMAP.md, mirroring crew/captain.rs,
// crew/fo.rs etc. clippy's module_inception lint exists to catch
// accidental name clashes that confuse readers, not deliberate ones --
// there's exactly one type here and it's unambiguous which "app" is meant.
#[allow(clippy::module_inception)]
pub mod app;
