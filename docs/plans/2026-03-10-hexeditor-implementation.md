# HexEditor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 基于 `gpui` 和 `gpui_component` 实现一个可复用的现代简洁风 HexEditor 组件库，并提供基础 demo。

**Architecture:** 采用“核心状态模型 + 组合式视图”的结构。先通过测试驱动实现偏移换算、光标移动、选择与编辑逻辑，再将逻辑接入 `gpui` 组件树，最后补齐主题样式、工具栏、状态栏和 demo。

**Tech Stack:** Rust、`gpui`、`gpui_component`、`cargo test`、`cargo check`

---

### Task 1: 初始化 crate 结构

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/config.rs`
- Create: `src/state.rs`
- Create: `src/style.rs`
- Create: `src/widget.rs`
- Create: `examples/basic.rs`

**Step 1: 建立最小 failing test 入口**

在 `src/state.rs` 内先写最小单元测试骨架，声明一个最简单的行列换算测试。

**Step 2: 运行测试确认失败**

Run: `cargo test state::tests::maps_offset_to_row_and_column -- --exact`
Expected: FAIL，提示类型或函数尚未实现

**Step 3: 写最小实现骨架**

- 建立 `HexEditorConfig`
- 建立 `HexEditorState`
- 建立 `HexEditor` 导出入口
- 让 crate 可以被编译

**Step 4: 运行测试确认通过或进入下一失败点**

Run: `cargo test state::tests::maps_offset_to_row_and_column -- --exact`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs src/config.rs src/state.rs src/style.rs src/widget.rs examples/basic.rs
git commit -m "feat: 初始化 hexeditor 组件库结构"
```

### Task 2: 实现偏移映射与光标移动逻辑

**Files:**
- Modify: `src/state.rs`
- Modify: `src/config.rs`
- Test: `src/state.rs`

**Step 1: 写 failing tests**

增加以下测试：
- 偏移映射到行列
- 左右上下移动光标
- `Home` / `End` 移动到行首行尾
- 边界位置不越界

**Step 2: 运行测试确认失败**

Run: `cargo test state::tests -- --nocapture`
Expected: FAIL，且失败原因是移动逻辑尚未实现或实现不完整

**Step 3: 写最小实现**

实现：
- `offset_to_row_col`
- `row_col_to_offset`
- `move_left`
- `move_right`
- `move_up`
- `move_down`
- `move_home`
- `move_end`

**Step 4: 运行测试确认通过**

Run: `cargo test state::tests -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/state.rs src/config.rs
git commit -m "feat: 实现光标移动与偏移换算"
```

### Task 3: 实现选区逻辑

**Files:**
- Modify: `src/state.rs`
- Test: `src/state.rs`

**Step 1: 写 failing tests**

增加以下测试：
- 单点定位清除旧选区
- 拖拽扩展连续选区
- 反向选择时仍输出规范化范围
- 跨行选择长度正确

**Step 2: 运行测试确认失败**

Run: `cargo test selection -- --nocapture`
Expected: FAIL，原因是选区逻辑未实现

**Step 3: 写最小实现**

实现：
- `set_cursor`
- `begin_selection`
- `update_selection`
- `clear_selection`
- `selected_range`
- `selection_len`

**Step 4: 运行测试确认通过**

Run: `cargo test selection -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/state.rs
git commit -m "feat: 实现字节选区逻辑"
```

### Task 4: 实现 Hex 与 ASCII 编辑逻辑

**Files:**
- Modify: `src/state.rs`
- Test: `src/state.rs`

**Step 1: 写 failing tests**

增加以下测试：
- Hex 半字节第一次输入只记录 pending 状态
- 第二次输入后写入完整字节
- ASCII 输入写入可见字符
- 只读模式下写入失败
- 写入后 dirty 置为 true

**Step 2: 运行测试确认失败**

Run: `cargo test edit -- --nocapture`
Expected: FAIL，原因是编辑逻辑未实现

**Step 3: 写最小实现**

实现：
- `apply_hex_digit`
- `apply_ascii_char`
- `set_read_only`
- `is_dirty`
- `pending_nibble`

**Step 4: 运行测试确认通过**

Run: `cargo test edit -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/state.rs
git commit -m "feat: 实现 hex 与 ascii 编辑逻辑"
```

### Task 5: 接入主题样式模型

**Files:**
- Modify: `src/style.rs`
- Modify: `src/lib.rs`
- Modify: `src/widget.rs`

**Step 1: 写 failing test 或最小可验证断言**

如果 `gpui` / `gpui_component` 主题对象适合做纯逻辑测试，则为样式派生函数编写单元测试；否则定义最小构造接口并通过 `cargo check` 作为验证。

**Step 2: 运行验证确认失败**

Run: `cargo check`
Expected: FAIL，原因是样式接口未接入或类型不匹配

**Step 3: 写最小实现**

实现：
- `HexEditorStyle`
- 从 `gpui_component` theme 派生背景、边框、文本、hover、selection、cursor 等语义色
- 默认间距、单元格尺寸、圆角参数

**Step 4: 运行验证确认通过**

Run: `cargo check`
Expected: PASS

**Step 5: Commit**

```bash
git add src/style.rs src/lib.rs src/widget.rs
git commit -m "feat: 接入 hexeditor 主题样式模型"
```

### Task 6: 实现基础视图布局

**Files:**
- Modify: `src/widget.rs`
- Modify: `src/lib.rs`
- Test: `cargo check`

**Step 1: 写 failing 验证目标**

定义目标布局：工具栏、地址栏、hex 网格、ascii 网格、状态栏都存在对应渲染函数或视图构建入口。

**Step 2: 运行验证确认失败**

Run: `cargo check`
Expected: FAIL，原因是组件结构未实现

**Step 3: 写最小实现**

实现：
- `HexEditor` 组件主体
- Toolbar 区域
- Address Gutter
- Hex Grid
- ASCII Grid
- Status Bar

**Step 4: 运行验证确认通过**

Run: `cargo check`
Expected: PASS

**Step 5: Commit**

```bash
git add src/widget.rs src/lib.rs
git commit -m "feat: 实现 hexeditor 基础布局"
```

### Task 7: 接入键盘与鼠标交互

**Files:**
- Modify: `src/widget.rs`
- Modify: `src/state.rs`
- Test: `src/state.rs`

**Step 1: 写 failing tests**

补充状态测试覆盖：
- 鼠标点击映射到 offset
- 拖拽更新选区
- 键盘方向键联动状态更新
- 模式切换影响输入路径

**Step 2: 运行测试确认失败**

Run: `cargo test --lib`
Expected: FAIL，原因是交互状态尚未完整接通

**Step 3: 写最小实现**

实现 widget 事件分发：
- 点击聚焦
- 单元格点击定位
- 拖拽选区
- 键盘移动
- Hex / ASCII 输入分发

**Step 4: 运行测试确认通过**

Run: `cargo test --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/widget.rs src/state.rs
git commit -m "feat: 实现 hexeditor 键鼠交互"
```

### Task 8: 完成工具栏、状态栏与示例

**Files:**
- Modify: `src/widget.rs`
- Modify: `examples/basic.rs`
- Modify: `README.md`

**Step 1: 写 failing 验证目标**

定义 demo 需展示：
- 初始数据加载
- 每行字节数切换
- 跳转地址
- 状态栏展示当前偏移与选区长度

**Step 2: 运行验证确认失败**

Run: `cargo check --examples`
Expected: FAIL，原因是示例或控件入口未完成

**Step 3: 写最小实现**

实现：
- 工具栏控件
- 状态栏文本
- `examples/basic.rs` 演示应用
- `README.md` 简短用法说明

**Step 4: 运行验证确认通过**

Run: `cargo check --examples`
Expected: PASS

**Step 5: Commit**

```bash
git add src/widget.rs examples/basic.rs README.md
git commit -m "feat: 完成 hexeditor 示例与说明"
```

### Task 9: 最终验证

**Files:**
- Verify: `src/*.rs`
- Verify: `examples/basic.rs`
- Verify: `README.md`

**Step 1: 运行库测试**

Run: `cargo test --lib`
Expected: PASS

**Step 2: 运行示例检查**

Run: `cargo check --examples`
Expected: PASS

**Step 3: 运行完整检查**

Run: `cargo check`
Expected: PASS

**Step 4: 整理代码与文档**

确保导出入口与说明一致，删除明显死代码，保持最小可维护结构。

**Step 5: Commit**

```bash
git add Cargo.toml src examples README.md docs/plans
git commit -m "feat: 完成 hexeditor 组件首版实现"
```
