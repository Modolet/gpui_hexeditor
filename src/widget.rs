use gpui::{
    App, ClipboardItem, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, MouseMoveEvent, MouseUpEvent, ParentElement, Render, SharedString,
    Styled, Window, div, prelude::FluentBuilder,
};
use gpui_component::{ActiveTheme, label::Label};

use crate::{HexEditorConfig, HexEditorState, HexEditorStyle, state::ActivePane};

pub struct HexEditor {
    state: HexEditorState,
    config: HexEditorConfig,
    style: Option<HexEditorStyle>,
    title: SharedString,
    focus_handle: FocusHandle,
    dragging_selection: bool,
}

impl HexEditor {
    pub fn new(buffer: Vec<u8>, config: HexEditorConfig, cx: &mut Context<Self>) -> Self {
        let mut state = HexEditorState::new(buffer, config.bytes_per_row);
        state.set_read_only(config.read_only);
        Self::from_state(state, config, cx)
    }

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

    pub fn bytes_cloned(&self) -> Vec<u8> {
        self.state.bytes_cloned()
    }

    pub fn len(&self) -> usize {
        self.state.len()
    }

    pub fn cursor_offset(&self) -> usize {
        self.state.cursor_offset()
    }

    pub fn overwrite_byte(&mut self, value: u8) {
        let _ = self.state.overwrite_byte(value);
    }

    pub fn apply_ascii_char(&mut self, character: char) {
        let _ = self.state.apply_ascii_char(character);
    }

    pub fn move_left(&mut self) {
        self.state.move_left();
    }

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
        window.focus(&self.focus_handle);
        cx.notify();
    }

    fn update_pointer_selection(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.dragging_selection {
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
        window.focus(&self.focus_handle);
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
            && event.keystroke.key.eq_ignore_ascii_case("a")
        {
            self.state.select_all();
            cx.notify();
            return;
        }

        if (modifiers.control || modifiers.platform)
            && event.keystroke.key.eq_ignore_ascii_case("v")
        {
            if let Some(item) = cx.read_from_clipboard() {
                if let Some(text) = item.text() {
                    if self.state.paste_hex_string(&text).is_ok() {
                        cx.notify();
                    }
                }
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
            if self.state.handle_text_input(text) {
                cx.notify();
                return;
            }
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

    fn render_hex_cell(
        &self,
        offset: usize,
        value: String,
        style: HexEditorStyle,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_cursor = offset == self.state.cursor_offset();
        let is_selected = self.state.is_selected(offset);
        let cell = div()
            .id(("hex", offset))
            .w_10()
            .px_0p5()
            .py_0p5()
            .text_center()
            .font_family("DejaVu Sans Mono")
            .text_xs()
            .rounded(style.cell_radius)
            .text_color(style.text)
            .cursor_pointer()
            .when(is_selected, |this| this.bg(style.selection_background))
            .when(is_cursor, |this| {
                this.bg(style.current_cell_background)
                    .border_1()
                    .border_color(style.focus_ring)
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                    if event.click_count >= 2 {
                        let row = this.state.offset_to_row_col(offset).0;
                        this.state.set_active_pane(ActivePane::Hex);
                        this.state.select_row(row);
                        window.focus(&this.focus_handle);
                        cx.notify();
                        return;
                    }
                    if event.modifiers.shift {
                        this.shift_click_select(offset, ActivePane::Hex, window, cx);
                    } else {
                        this.begin_pointer_selection(offset, ActivePane::Hex, false, window, cx);
                    }
                }),
            );
        let cell = cell.on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _, cx| {
            if event.dragging() {
                this.update_pointer_selection(offset, cx);
            }
        }));

        let cell = if is_selected {
            cell.bg(style.selection_background)
        } else {
            cell
        };

        let cell = if is_cursor {
            cell.bg(style.current_cell_background)
                .border_1()
                .border_color(style.focus_ring)
        } else {
            cell
        };

        cell.child(value)
    }

    fn render_ascii_cell(
        &self,
        offset: usize,
        ascii: char,
        style: HexEditorStyle,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_cursor = offset == self.state.cursor_offset();
        let is_selected = self.state.is_selected(offset);
        let cell = div()
            .id(("ascii", offset))
            .w_3()
            .text_center()
            .font_family("DejaVu Sans Mono")
            .text_xs()
            .rounded(style.cell_radius)
            .cursor_pointer()
            .when(is_selected, |this| this.bg(style.selection_background))
            .when(is_cursor, |this| {
                this.bg(style.current_cell_background)
                    .border_1()
                    .border_color(style.focus_ring)
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                    if event.click_count >= 2 {
                        let row = this.state.offset_to_row_col(offset).0;
                        this.state.set_active_pane(ActivePane::Ascii);
                        this.state.select_row(row);
                        window.focus(&this.focus_handle);
                        cx.notify();
                        return;
                    }
                    if event.modifiers.shift {
                        this.shift_click_select(offset, ActivePane::Ascii, window, cx);
                    } else {
                        this.begin_pointer_selection(offset, ActivePane::Ascii, false, window, cx);
                    }
                }),
            );
        let cell = cell.on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _, cx| {
            if event.dragging() {
                this.update_pointer_selection(offset, cx);
            }
        }));

        let cell = if is_selected {
            cell.bg(style.selection_background)
        } else {
            cell
        };

        let cell = if is_cursor {
            cell.bg(style.current_cell_background)
                .border_1()
                .border_color(style.focus_ring)
        } else {
            cell
        };

        cell.child(ascii.to_string())
    }

    fn render_row(
        &self,
        row: usize,
        style: HexEditorStyle,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let bytes = self.state.row_bytes(row);
        let row_start = row * self.state.bytes_per_row();
        let is_current_row = self.state.offset_to_row_col(self.state.cursor_offset()).0 == row;
        let mut hex_cells = Vec::with_capacity(self.state.bytes_per_row());
        for column in 0..self.state.bytes_per_row() {
            let offset = row_start + column;
            let value = bytes
                .get(column)
                .map(|byte| format!("{byte:02X}"))
                .unwrap_or_else(|| "  ".to_string());
            hex_cells.push(
                self.render_hex_cell(offset, value, style, cx)
                    .into_any_element(),
            );
        }

        let mut ascii_cells = Vec::with_capacity(bytes.len());
        for (column, byte) in bytes.iter().enumerate() {
            let offset = row_start + column;
            let ascii = if byte.is_ascii_graphic() || *byte == b' ' {
                char::from(*byte)
            } else {
                '·'
            };
            ascii_cells.push(
                self.render_ascii_cell(offset, ascii, style, cx)
                    .into_any_element(),
            );
        }

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
            .child(div().flex().flex_1().gap_0p5().children(hex_cells))
            .when(self.config.show_ascii, |this| {
                this.child(div().w_40().flex().gap_0p5().children(ascii_cells))
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
                "Offset 0x{:08X} · 行 {} 列 {}",
                self.state.cursor_offset(),
                row + 1,
                col + 1
            ))
            .child(format!(
                "{} · 选中 {} B · {}",
                self.state
                    .selected_range_label()
                    .unwrap_or_else(|| "无选区".to_string()),
                self.state.selection_len(),
                if self.state.is_dirty() {
                    "已修改"
                } else {
                    "未修改"
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
        let visible_rows = self.config.visible_rows.min(self.state.row_count().max(1));
        let current_row = self.state.offset_to_row_col(self.state.cursor_offset()).0;
        let start_row = current_row.saturating_sub(visible_rows / 2);
        let end_row = (start_row + visible_rows).min(self.state.row_count());
        let is_focused = self.focus_handle.is_focused(window);
        let mut rows = Vec::with_capacity(end_row.saturating_sub(start_row));
        for row in start_row..end_row {
            rows.push(self.render_row(row, style, cx).into_any_element());
        }

        let root = div()
            .id("hex-editor")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .flex()
            .flex_col()
            .bg(style.background)
            .border_1()
            .rounded(style.radius)
            .overflow_hidden();

        let mut root = root.border_color(if is_focused {
            style.focus_ring
        } else {
            style.border
        });
        if self.config.show_toolbar {
            root = root.child(self.render_toolbar(style));
        }
        root = root.child(self.render_header(style));
        root = root.child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .p_1()
                .bg(cx.theme().background)
                .children(rows),
        );
        if self.config.show_status_bar {
            root = root.child(self.render_status_bar(style));
        }
        root
    }
}

pub type HexEditorEntity = Entity<HexEditor>;
