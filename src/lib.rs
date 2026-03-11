//! `gpui_hexeditor` provides a reusable hex editor widget for GPUI applications.
//!
//! It includes:
//! - a configurable `HexEditor` widget
//! - a pure state model for editing and selections
//! - theme-aware styling helpers
//!
//! The widget can be used either as a fixed-height editor via `visible_rows`,
//! or as a panel-filling editor via `fill_container`.

pub mod config;
pub mod state;
pub mod style;
pub mod widget;

pub use config::HexEditorConfig;
pub use state::{ActivePane, EditError, HexEditorState, SelectionRange};
pub use style::HexEditorStyle;
pub use widget::{HexEditor, HexEditorEntity};
