mod row;

#[cfg(test)]
mod tests;

use gpui_kit::component::{ActiveTheme, VirtualListScrollHandle, label::Label, v_virtual_list};
use gpui_kit::{
    App, ClipboardItem, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, MouseMoveEvent, MouseUpEvent, ParentElement, Render, SharedString,
    Styled, Window, div, prelude::FluentBuilder, px,
};
use std::rc::Rc;

use crate::{HexEditorConfig, HexEditorState, HexEditorStyle, state::ActivePane};

/// A reusable GPUI hex editor widget.
pub struct HexEditor {
    state: HexEditorState,
    config: HexEditorConfig,
    style: Option<HexEditorStyle>,
    title: SharedString,
    focus_handle: FocusHandle,
    dragging_selection: bool,
    scroll_handle: VirtualListScrollHandle,
}

impl HexEditor {
    /// 创建一个新的可编辑 `HexEditor` 组件。
    pub fn new(buffer: Vec<u8>, config: HexEditorConfig, cx: &mut Context<Self>) -> Self {
        let mut state = HexEditorState::new(buffer, config.bytes_per_row);
        state.set_read_only(config.read_only);
        Self::from_state(state, config, cx)
    }

    /// Create an editor from an existing state object.
    pub fn from_state(
        state: HexEditorState,
        config: HexEditorConfig,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            state,
            config,
            style: None,
            title: "Hex Editor".into(),
            focus_handle: cx.focus_handle(),
            dragging_selection: false,
            scroll_handle: VirtualListScrollHandle::new(),
        }
    }

    /// Set the title shown in the optional toolbar.
    pub fn with_title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    /// Override the resolved theme-based style.
    pub fn with_style(mut self, style: HexEditorStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// 将当前内容标记为已保存，清空脏字节高亮。
    pub fn mark_saved(&mut self) {
        self.state.mark_saved();
    }

    /// 撤销最近一次编辑操作。
    pub fn undo(&mut self) -> bool {
        self.state.undo()
    }

    /// 重做最近一次撤销操作。
    pub fn redo(&mut self) -> bool {
        self.state.redo()
    }

    /// Return a cloned snapshot of the current buffer.
    pub fn bytes_cloned(&self) -> Vec<u8> {
        self.state.bytes_cloned()
    }

    /// Replace the entire buffer and reset dirty/undo state.
    pub fn set_bytes(&mut self, bytes: Vec<u8>) {
        self.state.replace_bytes(bytes);
        if !self.config.show_ascii && self.state.active_pane() == ActivePane::Ascii {
            self.state.set_active_pane(ActivePane::Hex);
        }
    }

    /// Return whether the current buffer differs from the saved baseline.
    pub fn is_dirty(&self) -> bool {
        self.state.is_dirty()
    }

    /// Return the buffer length in bytes.
    pub fn len(&self) -> usize {
        self.state.len()
    }

    /// Returns whether the editor contains no bytes.
    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
    }

    /// Return the current cursor offset.
    pub fn cursor_offset(&self) -> usize {
        self.state.cursor_offset()
    }

    /// Overwrite the byte at the current cursor position.
    pub fn overwrite_byte(&mut self, value: u8) {
        let _ = self.state.overwrite_byte(value);
    }

    /// Apply an ASCII character at the current cursor position when ASCII is enabled.
    pub fn apply_ascii_char(&mut self, character: char) {
        if self.config.show_ascii {
            let _ = self.state.apply_ascii_char(character);
        }
    }

    /// Move the cursor one byte to the left.
    pub fn move_left(&mut self) {
        self.state.move_left();
    }

    /// Move the cursor one byte to the right.
    pub fn move_right(&mut self) {
        self.state.move_right();
    }

    fn resolved_style(&self, cx: &App) -> HexEditorStyle {
        self.style.unwrap_or_else(|| HexEditorStyle::resolve(cx))
    }

    fn begin_pointer_selection(
        &mut self,
        offset: usize,
        pane: ActivePane,
        extend: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.state.set_active_pane(pane);
        if extend {
            let anchor = self
                .state
                .selection_anchor()
                .unwrap_or(self.state.cursor_offset());
            self.state.begin_selection(anchor);
            self.state.update_selection(offset);
        } else {
            self.state.begin_selection(offset);
        }
        self.dragging_selection = true;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    fn update_pointer_selection(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.dragging_selection && self.state.cursor_offset() != offset {
            self.state.update_selection(offset);
            cx.notify();
        }
    }

    fn shift_click_select(
        &mut self,
        offset: usize,
        pane: ActivePane,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.state.set_active_pane(pane);
        let anchor = self
            .state
            .selection_anchor()
            .unwrap_or(self.state.cursor_offset());
        self.state.begin_selection(anchor);
        self.state.update_selection(offset);
        self.dragging_selection = false;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    fn extend_selection_with_key(&mut self, key: &str) -> bool {
        let anchor = self
            .state
            .selection_anchor()
            .unwrap_or(self.state.cursor_offset());
        self.state.begin_selection(anchor);
        let handled = self.state.handle_key(key, self.config.visible_rows);
        if handled {
            let focus = self.state.cursor_offset();
            self.state.update_selection(focus);
        }
        handled
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let modifiers = event.keystroke.modifiers;
        if (modifiers.control || modifiers.platform)
            && event.keystroke.key.eq_ignore_ascii_case("c")
        {
            if let Some(text) = self.state.selected_hex_string() {
                cx.write_to_clipboard(ClipboardItem::new_string(text));
            }
            return;
        }

        if (modifiers.control || modifiers.platform)
            && event.keystroke.key.eq_ignore_ascii_case("x")
        {
            if let Ok(Some(text)) = self.state.cut_selection_hex() {
                cx.write_to_clipboard(ClipboardItem::new_string(text));
                cx.notify();
            }
            return;
        }

        if (modifiers.control || modifiers.platform)
            && event.keystroke.key.eq_ignore_ascii_case("z")
        {
            let changed = if modifiers.shift {
                self.state.redo()
            } else {
                self.state.undo()
            };
            if changed {
                cx.notify();
            }
            return;
        }

        if (modifiers.control || modifiers.platform)
            && event.keystroke.key.eq_ignore_ascii_case("y")
        {
            if self.state.redo() {
                cx.notify();
            }
            return;
        }

        if (modifiers.control || modifiers.platform)
            && event.keystroke.key.eq_ignore_ascii_case("a")
        {
            self.state.select_all();
            cx.notify();
            return;
        }

        if (modifiers.control || modifiers.platform)
            && event.keystroke.key.eq_ignore_ascii_case("v")
        {
            if let Some(item) = cx.read_from_clipboard()
                && let Some(text) = item.text()
                && self.state.paste_hex_string(&text).is_ok()
            {
                cx.notify();
            }
            return;
        }

        if modifiers.shift && self.extend_selection_with_key(event.keystroke.key.as_str()) {
            cx.notify();
            return;
        }

        let handled_navigation = self
            .state
            .handle_key(event.keystroke.key.as_str(), self.config.visible_rows);
        if handled_navigation {
            cx.notify();
            return;
        }

        if modifiers.control || modifiers.alt || modifiers.platform || modifiers.function {
            return;
        }

        if let Some(text) = event.keystroke.key_char.as_deref() {
            if !self.config.show_ascii && self.state.active_pane() == ActivePane::Ascii {
                self.state.set_active_pane(ActivePane::Hex);
            }
            if self.state.handle_text_input(text) {
                cx.notify();
                return;
            }
        }

        if !self.config.show_ascii && self.state.active_pane() == ActivePane::Ascii {
            self.state.set_active_pane(ActivePane::Hex);
        }
        if self.state.handle_text_input(event.keystroke.key.as_str()) {
            cx.notify();
        }
    }

    fn on_mouse_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.dragging_selection = false;
    }

    fn render_toolbar(&self, style: HexEditorStyle) -> impl IntoElement {
        let mode = if self.config.read_only {
            "Read Only"
        } else {
            "Editable"
        };
        div()
            .flex()
            .flex_row()
            .justify_between()
            .items_center()
            .px_1p5()
            .py_0p5()
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
                            "{} B · {} / row",
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
                    .gap_1()
                    .text_xs()
                    .text_color(style.muted_text)
                    .child(format!("Mode {mode}"))
                    .child(format!(
                        "View {}",
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
            .gap_1p5()
            .px_1p5()
            .py_0p5()
            .border_b_1()
            .border_color(style.border.opacity(0.65))
            .bg(style.gutter_background)
            .child(
                div()
                    .w(px(72.))
                    .flex_shrink_0()
                    .text_xs()
                    .text_color(style.muted_text)
                    .child("Offset"),
            )
            .child(div().min_w_0().flex_1().flex().gap_0p5().children(
                (0..self.state.bytes_per_row()).map(|index| {
                    div()
                        .w(px(36.))
                        .text_center()
                        .text_xs()
                        .text_color(style.muted_text)
                        .child(format!("{index:02X}"))
                }),
            ))
            .when(self.config.show_ascii, |this| {
                this.child(
                    div()
                        .w(px(144.))
                        .flex_shrink_0()
                        .text_xs()
                        .text_color(style.muted_text)
                        .child("ASCII"),
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
            .px_1p5()
            .py_0p5()
            .rounded_b(style.radius)
            .border_t_1()
            .border_color(style.border)
            .bg(style.panel_background)
            .text_xs()
            .child(format!(
                "Offset 0x{:08X} · Row {} Col {}",
                self.state.cursor_offset(),
                row + 1,
                col + 1
            ))
            .child(format!(
                "{} · Selected {} B · {}",
                self.state
                    .selected_range_label()
                    .unwrap_or_else(|| "No Selection".to_string()),
                self.state.selection_len(),
                if self.state.is_dirty() {
                    "Modified"
                } else {
                    "Clean"
                }
            ))
    }
}

impl Focusable for HexEditor {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for HexEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let style = self.resolved_style(cx);
        let is_focused = self.focus_handle.is_focused(window);
        let row_count = self.state.row_count();
        let row_sizes = Rc::new(
            (0..row_count)
                .map(|_| gpui_kit::size(px(0.), style.row_height))
                .collect::<Vec<_>>(),
        );
        let visible_rows = self.config.visible_rows.min(row_count.max(1));
        let content_height = style.row_height * visible_rows as f32 + style.section_gap * 2.0;

        let mut root = div()
            .id("hex-editor")
            .w_full()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .flex()
            .flex_col()
            .bg(style.background)
            .border_1()
            .rounded(style.radius)
            .overflow_hidden();
        if self.config.fill_container {
            root = root.h_full().min_h_0();
        }

        let mut root = root.border_color(if is_focused {
            style.focus_ring
        } else {
            style.border
        });
        if self.config.show_toolbar {
            root = root.child(self.render_toolbar(style));
        }
        root = root.child(self.render_header(style));
        let mut scroll = div()
            .id("hex-editor-scroll")
            .overflow_hidden()
            .flex()
            .flex_col()
            .gap_0p5()
            .p_0p5()
            .bg(cx.theme().background)
            .child(
                v_virtual_list(
                    cx.entity(),
                    "hex-editor-rows",
                    row_sizes,
                    move |editor, visible_range, _window, cx| {
                        visible_range
                            .map(|row| editor.render_row(row, style, cx).into_any_element())
                            .collect::<Vec<_>>()
                    },
                )
                .track_scroll(&self.scroll_handle)
                .size_full(),
            );
        scroll = if self.config.fill_container {
            scroll.flex_1().min_h_0()
        } else {
            scroll.h(content_height)
        };
        root = root.child(scroll);
        if self.config.show_status_bar {
            root = root.child(self.render_status_bar(style));
        }
        root
    }
}

pub type HexEditorEntity = Entity<HexEditor>;
