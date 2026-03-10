use gpui::{
    App, Application, Bounds, Context, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, size,
};
use gpui_component::ActiveTheme;
use gpui_hexeditor::{HexEditor, HexEditorConfig};

struct Demo;

impl Render for Demo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sample = (0u8..=255).collect::<Vec<_>>();
        let editor = HexEditor::new(
            sample,
            HexEditorConfig::default()
                .with_visible_rows(16)
                .with_bytes_per_row(16),
        )
        .with_title("HexEditor Demo");

        div()
            .size_full()
            .p_3()
            .bg(cx.theme().background)
            .child(editor)
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(1200.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Demo),
        )
        .unwrap();

        cx.activate(true);
    });
}
