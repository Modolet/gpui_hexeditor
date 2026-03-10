use gpui::{
    App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{ActiveTheme, label::Label};

use crate::{HexEditorConfig, HexEditorState, HexEditorStyle, state::ActivePane};

#[derive(Debug, Clone, IntoElement)]
pub struct HexEditor {
    pub state: HexEditorState,
    pub config: HexEditorConfig,
    pub style: Option<HexEditorStyle>,
    title: SharedString,
}

impl HexEditor {
    pub fn new(buffer: Vec<u8>, config: HexEditorConfig) -> Self {
        let mut state = HexEditorState::new(buffer, config.bytes_per_row);
        state.set_read_only(config.read_only);
        Self::from_state(state, config)
    }

    pub fn from_state(state: HexEditorState, config: HexEditorConfig) -> Self {
        Self {
            state,
            config,
            style: None,
            title: "Hex Editor".into(),
        }
    }

    pub fn with_title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_style(mut self, style: HexEditorStyle) -> Self {
        self.style = Some(style);
        self
    }

    fn resolved_style(&self, cx: &App) -> HexEditorStyle {
        self.style.unwrap_or_else(|| HexEditorStyle::resolve(cx))
    }

    fn render_toolbar(&self, style: HexEditorStyle) -> impl IntoElement {
        let mode = if self.config.read_only {
            "只读"
        } else {
            "编辑"
        };
        div()
            .flex()
            .flex_row()
            .justify_between()
            .items_center()
            .px_2()
            .py_1()
            .rounded_t(style.radius)
            .border_b_1()
            .border_color(style.border)
            .bg(style.panel_background)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(Label::new(self.title.clone()))
                    .child(
                        Label::new(format!(
                            "{} B · {} / 行",
                            self.state.len(),
                            self.state.bytes_per_row()
                        ))
                        .text_sm(),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_xs()
                    .text_color(style.muted_text)
                    .child(format!("模式 {mode}"))
                    .child(format!(
                        "视图 {}",
                        match self.state.active_pane() {
                            ActivePane::Hex => "Hex",
                            ActivePane::Ascii => "ASCII",
                        }
                    )),
            )
    }

    fn render_header(&self, style: HexEditorStyle) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(style.border.opacity(0.65))
            .bg(style.gutter_background)
            .child(
                div()
                    .w_20()
                    .text_xs()
                    .text_color(style.muted_text)
                    .child("Offset"),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .gap_0p5()
                    .children((0..self.state.bytes_per_row()).map(|index| {
                        div()
                            .w_10()
                            .text_center()
                            .text_xs()
                            .text_color(style.muted_text)
                            .child(format!("{index:02X}"))
                    })),
            )
            .when(self.config.show_ascii, |this| {
                this.child(
                    div()
                        .w_40()
                        .text_xs()
                        .text_color(style.muted_text)
                        .child("ASCII"),
                )
            })
    }

    fn render_row(&self, row: usize, style: HexEditorStyle) -> impl IntoElement {
        let bytes = self.state.row_bytes(row);
        let row_start = row * self.state.bytes_per_row();
        let is_current_row = self.state.offset_to_row_col(self.state.cursor_offset()).0 == row;

        div()
            .flex()
            .flex_row()
            .items_center()
            .min_h(style.row_height)
            .px_2()
            .py_0p5()
            .gap_2()
            .when(is_current_row, |this| this.bg(style.current_row_background))
            .child(
                div()
                    .w_20()
                    .font_family("DejaVu Sans Mono")
                    .text_xs()
                    .text_color(style.muted_text)
                    .child(format!("{row_start:08X}")),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .gap_0p5()
                    .children((0..self.state.bytes_per_row()).map(|column| {
                        let offset = row_start + column;
                        let value = bytes
                            .get(column)
                            .map(|byte| format!("{byte:02X}"))
                            .unwrap_or_else(|| "  ".to_string());
                        let is_cursor = offset == self.state.cursor_offset();
                        let is_selected = self.state.is_selected(offset);
                        div()
                            .w_10()
                            .px_0p5()
                            .py_0p5()
                            .text_center()
                            .font_family("DejaVu Sans Mono")
                            .text_xs()
                            .rounded(style.cell_radius)
                            .text_color(style.text)
                            .when(is_selected, |this| this.bg(style.selection_background))
                            .when(is_cursor, |this| {
                                this.bg(style.current_cell_background)
                                    .border_1()
                                    .border_color(style.focus_ring)
                            })
                            .child(value)
                    })),
            )
            .when(self.config.show_ascii, |this| {
                this.child(
                    div()
                        .w_40()
                        .flex()
                        .gap_0p5()
                        .children(bytes.iter().enumerate().map(|(column, byte)| {
                            let offset = row_start + column;
                            let is_cursor = offset == self.state.cursor_offset();
                            let is_selected = self.state.is_selected(offset);
                            let ascii = if byte.is_ascii_graphic() || *byte == b' ' {
                                char::from(*byte)
                            } else {
                                '·'
                            };
                            div()
                                .w_3()
                                .text_center()
                                .font_family("DejaVu Sans Mono")
                                .text_xs()
                                .rounded(style.cell_radius)
                                .when(is_selected, |this| this.bg(style.selection_background))
                                .when(is_cursor, |this| {
                                    this.bg(style.current_cell_background)
                                        .border_1()
                                        .border_color(style.focus_ring)
                                })
                                .child(ascii.to_string())
                        })),
                )
            })
    }

    fn render_status_bar(&self, style: HexEditorStyle) -> impl IntoElement {
        let (row, col) = self.state.offset_to_row_col(self.state.cursor_offset());
        div()
            .flex()
            .flex_row()
            .justify_between()
            .items_center()
            .px_2()
            .py_1()
            .rounded_b(style.radius)
            .border_t_1()
            .border_color(style.border)
            .bg(style.panel_background)
            .text_xs()
            .child(format!(
                "Offset 0x{:08X}  ·  行 {} 列 {}",
                self.state.cursor_offset(),
                row + 1,
                col + 1
            ))
            .child(format!(
                "选中 {} B  ·  {}",
                self.state.selection_len(),
                if self.state.is_dirty() {
                    "已修改"
                } else {
                    "未修改"
                }
            ))
    }
}

impl RenderOnce for HexEditor {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let style = self.resolved_style(cx);
        let visible_rows = self.config.visible_rows.min(self.state.row_count().max(1));
        let current_row = self.state.offset_to_row_col(self.state.cursor_offset()).0;
        let start_row = current_row.saturating_sub(visible_rows / 2);
        let end_row = (start_row + visible_rows).min(self.state.row_count());

        div()
            .flex()
            .flex_col()
            .bg(style.background)
            .border_1()
            .border_color(style.border)
            .rounded(style.radius)
            .overflow_hidden()
            .when(self.config.show_toolbar, |this| {
                this.child(self.render_toolbar(style))
            })
            .child(self.render_header(style))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .p_1()
                    .bg(cx.theme().background)
                    .children((start_row..end_row).map(|row| self.render_row(row, style))),
            )
            .when(self.config.show_status_bar, |this| {
                this.child(self.render_status_bar(style))
            })
    }
}
