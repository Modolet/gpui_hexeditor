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
use gpui::{Context, Window};
use gpui_hexeditor::{HexEditor, HexEditorConfig};

fn build_editor(window: &mut Window, cx: &mut Context<MyView>) {
    let _editor = cx.new(|cx| {
        HexEditor::new(
            vec![0x48, 0x65, 0x78, 0x21],
            HexEditorConfig::default().with_bytes_per_row(16),
            cx,
        )
    });
}
```

## 示例

- 纯展示：`cargo run --example basic`
- 打开并保存本地文件：`cargo run --example file_editor -- ./sample.bin`

## 当前交互

- 点击 Hex/ASCII 单元格后获得焦点
- 鼠标拖拽可连续建立选区
- `Shift + 方向键/Home/End/PageUp/PageDown` 可扩展选区
- `Ctrl/Cmd + C` 复制选中字节的十六进制文本
- `Ctrl/Cmd + X` 剪切选区为十六进制文本并清零原字节
- `Ctrl/Cmd + A` 全选全部字节
- 双击 Hex/ASCII 单元格可选中整行
- 方向键、`Home`、`End`、`PageUp`、`PageDown` 移动光标
- Hex 区直接输入 `0-9A-F`，按半字节连续写入
- ASCII 区直接输入可见字符覆盖字节
- `Backspace` / `Delete` 直接清零当前字节
