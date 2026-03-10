use gpui::{
    App, Application, Bounds, Context, Entity, Render, StatefulInteractiveElement, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    label::Label,
};
use gpui_hexeditor::{HexEditor, HexEditorConfig, HexEditorEntity, HexEditorStyle};

struct Showcase {
    main_editor: HexEditorEntity,
    compact_editor: HexEditorEntity,
    readonly_editor: HexEditorEntity,
    wide_editor: HexEditorEntity,
    dense_no_ascii_editor: HexEditorEntity,
    main_config: HexEditorConfig,
}

impl Showcase {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let main_config = HexEditorConfig::default()
            .with_visible_rows(20)
            .with_bytes_per_row(16);

        Self {
            main_editor: Self::build_main_editor(cx, main_config),
            compact_editor: Self::build_compact_editor(cx),
            readonly_editor: Self::build_readonly_editor(cx),
            wide_editor: Self::build_wide_editor(cx),
            dense_no_ascii_editor: Self::build_dense_no_ascii_editor(cx),
            main_config,
        }
    }

    fn sample_bytes() -> Vec<u8> {
        Self::sample_bytes_with_limit(224)
    }

    fn sample_bytes_with_limit(limit: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"GPUI HEXEDITOR SHOWCASE\0");
        bytes.extend(0u8..=63);
        bytes.extend_from_slice(b"Type hex digits, ASCII chars, copy/paste hex, undo/redo.\n");
        bytes.extend((0..128).map(|index| (index as u8).wrapping_mul(3)));
        bytes.truncate(limit);
        bytes
    }

    fn build_main_editor(cx: &mut Context<Self>, config: HexEditorConfig) -> Entity<HexEditor> {
        let bytes = Self::sample_bytes();
        cx.new(|cx| HexEditor::new(bytes, config, cx).with_title("Interactive Showcase"))
    }

    fn build_compact_editor(cx: &mut Context<Self>) -> Entity<HexEditor> {
        let style = HexEditorStyle::from_theme(cx.theme());
        let compact_style = HexEditorStyle {
            row_height: px(17.),
            cell_radius: px(2.),
            section_gap: px(4.),
            mono_font_size: px(10.),
            ..style
        };

        cx.new(|cx| {
            HexEditor::new(
                Self::sample_bytes_with_limit(72),
                HexEditorConfig::default()
                    .with_visible_rows(5)
                    .with_bytes_per_row(16),
                cx,
            )
            .with_title("Compact Style")
            .with_style(compact_style)
        })
    }

    fn build_readonly_editor(cx: &mut Context<Self>) -> Entity<HexEditor> {
        cx.new(|cx| {
            HexEditor::new(
                Self::sample_bytes_with_limit(64),
                HexEditorConfig::default()
                    .with_visible_rows(4)
                    .with_bytes_per_row(16)
                    .read_only(true),
                cx,
            )
            .with_title("Read Only")
        })
    }

    fn build_wide_editor(cx: &mut Context<Self>) -> Entity<HexEditor> {
        cx.new(|cx| {
            HexEditor::new(
                Self::sample_bytes_with_limit(96),
                HexEditorConfig::default()
                    .with_visible_rows(4)
                    .with_bytes_per_row(32),
                cx,
            )
            .with_title("32 Bytes / Row")
        })
    }

    fn build_dense_no_ascii_editor(cx: &mut Context<Self>) -> Entity<HexEditor> {
        let style = HexEditorStyle::from_theme(cx.theme());
        let custom_style = HexEditorStyle {
            dirty_cell_background: cx.theme().warning.opacity(0.22),
            dirty_cell_border: cx.theme().warning,
            current_row_background: cx.theme().info.opacity(0.18),
            ..style
        };

        cx.new(|cx| {
            let mut config = HexEditorConfig::default()
                .with_visible_rows(4)
                .with_bytes_per_row(12);
            config.show_ascii = false;
            HexEditor::new(Self::sample_bytes_with_limit(60), config, cx)
                .with_title("No ASCII + Custom Dirty Color")
                .with_style(custom_style)
        })
    }

    fn rebuild_main_editor(&mut self, cx: &mut Context<Self>) {
        self.main_editor = Self::build_main_editor(cx, self.main_config);
    }

    fn control_button(
        id: &'static str,
        label: &'static str,
        active: bool,
        on_click: impl Fn(&mut Showcase, &gpui::ClickEvent, &mut Window, &mut Context<Showcase>)
        + 'static,
        cx: &mut Context<Self>,
    ) -> Button {
        let button = Button::new(id).compact().label(label);
        let button = if active { button.primary() } else { button };
        button.on_click(cx.listener(on_click))
    }

    fn render_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_1p5()
            .p_2()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().secondary.opacity(0.32))
            .child(Label::new("Controls & Capabilities"))
            .child(
                Label::new(
                    "Try: click cells, type hex/ASCII, drag select, Ctrl/Cmd+C/X/V/A, Undo/Redo",
                )
                .text_sm(),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(Label::new("Bytes/Row").text_sm())
                    .child(Self::control_button(
                        "bpr-8",
                        "8",
                        self.main_config.bytes_per_row == 8,
                        |this, _, _, cx| {
                            this.main_config.bytes_per_row = 8;
                            this.rebuild_main_editor(cx);
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(Self::control_button(
                        "bpr-16",
                        "16",
                        self.main_config.bytes_per_row == 16,
                        |this, _, _, cx| {
                            this.main_config.bytes_per_row = 16;
                            this.rebuild_main_editor(cx);
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(Self::control_button(
                        "bpr-24",
                        "24",
                        self.main_config.bytes_per_row == 24,
                        |this, _, _, cx| {
                            this.main_config.bytes_per_row = 24;
                            this.rebuild_main_editor(cx);
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(Self::control_button(
                        "bpr-32",
                        "32",
                        self.main_config.bytes_per_row == 32,
                        |this, _, _, cx| {
                            this.main_config.bytes_per_row = 32;
                            this.rebuild_main_editor(cx);
                            cx.notify();
                        },
                        cx,
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(Label::new("Rows").text_sm())
                    .child(Self::control_button(
                        "rows-12",
                        "12",
                        self.main_config.visible_rows == 12,
                        |this, _, _, cx| {
                            this.main_config.visible_rows = 12;
                            this.rebuild_main_editor(cx);
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(Self::control_button(
                        "rows-20",
                        "20",
                        self.main_config.visible_rows == 20,
                        |this, _, _, cx| {
                            this.main_config.visible_rows = 20;
                            this.rebuild_main_editor(cx);
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(Self::control_button(
                        "rows-28",
                        "28",
                        self.main_config.visible_rows == 28,
                        |this, _, _, cx| {
                            this.main_config.visible_rows = 28;
                            this.rebuild_main_editor(cx);
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(Self::control_button(
                        "toggle-readonly",
                        if self.main_config.read_only {
                            "ReadOnly On"
                        } else {
                            "ReadOnly Off"
                        },
                        self.main_config.read_only,
                        |this, _, _, cx| {
                            this.main_config.read_only = !this.main_config.read_only;
                            this.rebuild_main_editor(cx);
                            cx.notify();
                        },
                        cx,
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .child(Self::control_button(
                        "toggle-ascii",
                        if self.main_config.show_ascii {
                            "ASCII On"
                        } else {
                            "ASCII Off"
                        },
                        self.main_config.show_ascii,
                        |this, _, _, cx| {
                            this.main_config.show_ascii = !this.main_config.show_ascii;
                            this.rebuild_main_editor(cx);
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(Self::control_button(
                        "toggle-toolbar",
                        if self.main_config.show_toolbar {
                            "Toolbar On"
                        } else {
                            "Toolbar Off"
                        },
                        self.main_config.show_toolbar,
                        |this, _, _, cx| {
                            this.main_config.show_toolbar = !this.main_config.show_toolbar;
                            this.rebuild_main_editor(cx);
                            cx.notify();
                        },
                        cx,
                    ))
                    .child(Self::control_button(
                        "toggle-status",
                        if self.main_config.show_status_bar {
                            "Status On"
                        } else {
                            "Status Off"
                        },
                        self.main_config.show_status_bar,
                        |this, _, _, cx| {
                            this.main_config.show_status_bar = !this.main_config.show_status_bar;
                            this.rebuild_main_editor(cx);
                            cx.notify();
                        },
                        cx,
                    )),
            )
    }

    fn render_variant_card(
        title: &'static str,
        description: &'static str,
        editor: Entity<HexEditor>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .p_2()
            .rounded_md()
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().secondary.opacity(0.18))
            .child(Label::new(title))
            .child(Label::new(description).text_sm())
            .child(editor)
    }
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .id("showcase-scroll")
            .overflow_y_scroll()
            .p_3()
            .gap_2()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(Label::new("HexEditor Showcase"))
                    .child(
                        Label::new(
                            "A configurable GPUI hex editor with direct byte editing, selection, clipboard actions, dirty-byte highlighting, undo/redo and internal scrolling.",
                        )
                        .text_sm(),
                    ),
            )
            .child(self.render_controls(cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(Label::new("Main Editor").text_sm())
                    .child(self.main_editor.clone()),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(Label::new("Variants").text_sm())
                    .child(
                        div()
                            .grid()
                            .grid_cols(2)
                            .gap_2()
                            .child(Self::render_variant_card(
                                "Compact",
                                "Denser typography and spacing for embedded layouts.",
                                self.compact_editor.clone(),
                                cx,
                            ))
                            .child(Self::render_variant_card(
                                "Read Only",
                                "Interaction, selection and scroll work, but editing is disabled.",
                                self.readonly_editor.clone(),
                                cx,
                            ))
                            .child(Self::render_variant_card(
                                "Wide Rows",
                                "Shows how bytes-per-row changes overall geometry.",
                                self.wide_editor.clone(),
                                cx,
                            ))
                            .child(Self::render_variant_card(
                                "Custom Style",
                                "ASCII hidden with customized dirty-byte warning colors.",
                                self.dense_no_ascii_editor.clone(),
                                cx,
                            )),
                    ),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(1520.), px(1080.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| Showcase::new(window, cx)),
        )
        .unwrap();

        cx.activate(true);
    });
}
