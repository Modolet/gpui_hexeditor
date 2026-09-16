# gpui_hexeditor

A reusable hex editor widget built with `gpui` and `gpui_kit::component`.

## Features

- Hex grid with optional ASCII pane
- Selection, drag selection, row selection
- Hex editing and ASCII editing
- Undo / redo
- Dirty byte tracking
- Internal scrolling with virtual rows and retained scroll position
- Batched HEX / ASCII row drawing with shared pointer geometry
- Theme-aware styling via `gpui_kit::component`
- Can either use fixed visible rows or fill the parent container height

## Installation

```toml
[dependencies]
gpui-kit = "0.6.0"
gpui_hexeditor = "0.2.2"
```

## Quick Start

```rust
use gpui_kit::Context;
use gpui_hexeditor::{HexEditor, HexEditorConfig};

fn build_editor(cx: &mut Context<MyView>) {
    let _editor = cx.new(|cx| {
        let mut config = HexEditorConfig::default().with_bytes_per_row(16);
        config.show_ascii = false;
        HexEditor::new(vec![0x48, 0x65, 0x78, 0x21], config, cx)
            .with_title("Hex Editor")
    });
}
```

## Configuration

`HexEditorConfig` supports:

- `bytes_per_row`
- `show_ascii`
- `show_toolbar`
- `show_status_bar`
- `read_only`
- `visible_rows`
- `fill_container`

Use `fill_container = true` when embedding the editor inside a panel that already controls height.

## Examples

```bash
cargo run --example basic
cargo run --example file_editor -- ./sample.bin
cargo run --example showcase
```

## Keyboard Shortcuts

- `Ctrl/Cmd + C`: copy selected bytes as hex text
- `Ctrl/Cmd + X`: cut selected bytes as hex text
- `Ctrl/Cmd + V`: paste hex text
- `Ctrl/Cmd + A`: select all
- `Ctrl/Cmd + Z`: undo
- `Ctrl/Cmd + Shift + Z`: redo
- `Home` / `End` / `PageUp` / `PageDown`: navigation
- `Backspace` / `Delete`: clear current byte

## License

MIT
