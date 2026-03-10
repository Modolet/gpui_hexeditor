# gpui_hexeditor

一个基于 `gpui` 和 `gpui_component` 的可复用 HexEditor 组件库。

## 当前特性

- 现代简洁风布局
- 兼容 `gpui_component` theme 的语义配色
- 地址栏、Hex 区、ASCII 区、状态栏
- 光标、选区、Hex/ASCII 编辑的状态模型
- 示例应用：`cargo run --example basic`
- 本地文件编辑示例：`cargo run --example file_editor -- /path/to/file.bin`

## 使用方式

```rust
use gpui_hexeditor::{HexEditor, HexEditorConfig};

let editor = HexEditor::new(
    vec![0x48, 0x65, 0x78, 0x21],
    HexEditorConfig::default().with_bytes_per_row(16),
);
```

## 示例

- 纯展示：`cargo run --example basic`
- 打开并保存本地文件：`cargo run --example file_editor -- ./sample.bin`
