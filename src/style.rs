use gpui::{App, Hsla, Pixels, px};
use gpui_component::{ActiveTheme, Theme};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HexEditorStyle {
    pub background: Hsla,
    pub panel_background: Hsla,
    pub gutter_background: Hsla,
    pub border: Hsla,
    pub text: Hsla,
    pub muted_text: Hsla,
    pub hover_background: Hsla,
    pub selection_background: Hsla,
    pub current_row_background: Hsla,
    pub current_cell_background: Hsla,
    pub focus_ring: Hsla,
    pub radius: Pixels,
    pub row_height: Pixels,
    pub cell_radius: Pixels,
    pub section_gap: Pixels,
    pub mono_font_size: Pixels,
}

impl HexEditorStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.background,
            panel_background: theme.secondary.opacity(0.42),
            gutter_background: theme.muted.opacity(0.56),
            border: theme.border,
            text: theme.foreground,
            muted_text: theme.muted_foreground,
            hover_background: theme.accent.opacity(0.55),
            selection_background: theme.selection.opacity(0.9),
            current_row_background: theme.accent.opacity(0.28),
            current_cell_background: theme.primary.opacity(0.16),
            focus_ring: theme.ring,
            radius: px(4.),
            row_height: px(19.),
            cell_radius: px(3.),
            section_gap: px(6.),
            mono_font_size: px(11.),
        }
    }

    pub fn resolve(cx: &App) -> Self {
        Self::from_theme(cx.theme())
    }
}
