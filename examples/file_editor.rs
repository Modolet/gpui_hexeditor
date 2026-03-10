use std::{env, fs, path::PathBuf};

use gpui::{
    App, Application, Bounds, Context, Render, SharedString, Window, WindowBounds, WindowOptions,
    div, prelude::*, px, size,
};
use gpui_component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
    label::Label,
};
use gpui_hexeditor::{HexEditor, HexEditorConfig, HexEditorEntity};

struct FileEditorDemo {
    path: PathBuf,
    editor: HexEditorEntity,
    message: SharedString,
}

impl FileEditorDemo {
    fn new(path: PathBuf, bytes: Vec<u8>, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        let title = path.display().to_string();
        let editor = cx.new(|cx| {
            HexEditor::new(
                bytes,
                HexEditorConfig::default()
                    .with_bytes_per_row(16)
                    .with_visible_rows(24),
                cx,
            )
            .with_title(title)
        });

        Self {
            path,
            editor,
            message: "已加载文件，点击单元格后直接键盘编辑".into(),
        }
    }

    fn save(&mut self, cx: &mut Context<Self>) -> std::io::Result<()> {
        let bytes = self.editor.read_with(cx, |editor, _| editor.bytes_cloned());
        fs::write(&self.path, bytes)?;
        self.editor.update(cx, |editor, cx| {
            editor.mark_saved();
            cx.notify();
        });
        self.message = format!("已保存到 {}", self.path.display()).into();
        Ok(())
    }
}

impl Render for FileEditorDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (len, offset) = self
            .editor
            .read_with(cx, |editor, _| (editor.len(), editor.cursor_offset()));

        div()
            .size_full()
            .p_3()
            .gap_2()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(Label::new(self.path.display().to_string()))
                            .child(
                                Label::new(format!("{} bytes · offset 0x{:08X}", len, offset))
                                    .text_sm(),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_1()
                            .child(
                                Button::new("left")
                                    .compact()
                                    .label("←")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.editor.update(cx, |editor, cx| {
                                            editor.move_left();
                                            cx.notify();
                                        });
                                    })),
                            )
                            .child(
                                Button::new("right")
                                    .compact()
                                    .label("→")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.editor.update(cx, |editor, cx| {
                                            editor.move_right();
                                            cx.notify();
                                        });
                                    })),
                            )
                            .child(
                                Button::new("zero")
                                    .compact()
                                    .label("00")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.editor.update(cx, |editor, cx| {
                                            editor.overwrite_byte(0x00);
                                            cx.notify();
                                        });
                                    })),
                            )
                            .child(
                                Button::new("ff")
                                    .compact()
                                    .label("FF")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.editor.update(cx, |editor, cx| {
                                            editor.overwrite_byte(0xFF);
                                            cx.notify();
                                        });
                                    })),
                            )
                            .child(Button::new("ascii-a").compact().label("写入 A").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.editor.update(cx, |editor, cx| {
                                        editor.apply_ascii_char('A');
                                        cx.notify();
                                    });
                                }),
                            ))
                            .child(
                                Button::new("save")
                                    .compact()
                                    .primary()
                                    .label("保存")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if let Err(error) = this.save(cx) {
                                            this.message = format!("保存失败: {error}").into();
                                        }
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .child(self.editor.clone())
            .child(
                div()
                    .px_1()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.message.clone()),
            )
    }
}

fn main() {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: cargo run --example file_editor -- <path>");
    let bytes = fs::read(&path).expect("failed to read file");

    Application::new().run(move |cx: &mut App| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(1240.), px(860.)), cx);
        let path = path.clone();
        let bytes = bytes.clone();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, cx| {
                let path = path.clone();
                let bytes = bytes.clone();
                cx.new(|cx| FileEditorDemo::new(path, bytes, window, cx))
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
