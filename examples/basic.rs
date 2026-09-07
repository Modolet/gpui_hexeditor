use gpui_hexeditor::{HexEditor, HexEditorConfig, HexEditorEntity};
use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    App, Bounds, Context, Render, Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};

struct Demo {
    editor: HexEditorEntity,
}

impl Demo {
    fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let sample = (0u8..=255).collect::<Vec<_>>();
        let editor = cx.new(|cx| {
            HexEditor::new(
                sample,
                HexEditorConfig::default()
                    .with_visible_rows(16)
                    .with_bytes_per_row(16),
                cx,
            )
            .with_title("HexEditor Demo")
        });

        Self { editor }
    }
}

impl Render for Demo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_3()
            .bg(cx.theme().background)
            .child(self.editor.clone())
    }
}

fn main() {
    gpui_kit::platform::application().run(|cx: &mut App| {
        gpui_kit::component::init(cx);

        let bounds = Bounds::centered(None, size(px(1200.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| Demo::new(window, cx)),
        )
        .unwrap();

        cx.activate(true);
    });
}
