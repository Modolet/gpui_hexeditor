# HexEditor 组件设计文档

## 目标

基于 `gpui` 与 `gpui_component` 构建一个可复用的 `HexEditor` 组件库，风格偏现代简洁，主题配色兼容 `gpui_component` 的 theme 体系。首版聚焦“标准编辑”能力，面向中等规模二进制数据的浏览与编辑。

## 范围

### 包含

- 字节地址栏展示
- Hex 区与 ASCII 区联动展示
- 键盘光标移动
- 鼠标单击与拖拽选择
- Hex 区编辑与 ASCII 区编辑
- 工具栏：跳转地址、每行字节数切换、只读/编辑状态
- 状态栏：当前偏移、选区长度、编辑模式
- 与 `gpui_component` theme 兼容的样式系统

### 不包含

- 搜索
- 书签
- 数据解释器
- 撤销/重做
- 超大文件 mmap / 流式加载

## 方案选择

采用“核心模型 + 组合式视图”的结构。

### 选择原因

- 对外可直接提供开箱即用的 `HexEditor` 主组件
- 内部可拆分为多个子区域，便于后续扩展
- 能兼顾复用性、可维护性和视觉完成度

## 架构设计

### 组件结构

- `HexEditor`：对外主组件
- `HexEditorState`：维护缓冲区、光标、选区、编辑模式、脏状态
- `HexEditorStyle`：封装主题语义色与尺寸参数
- 视图分区：
  - Toolbar
  - Address Gutter
  - Hex Grid
  - ASCII Grid
  - Status Bar

### 数据模型

首版采用内存字节缓冲区：`Vec<u8>`。

状态层维护：

- `cursor_offset`
- `selection: Option<RangeInclusive<usize>>`
- `bytes_per_row`
- `read_only`
- `dirty`
- `active_pane`（Hex / ASCII）
- `pending_nibble`（Hex 半字节输入）

### 渲染策略

- 以“行”为基本渲染单位
- 每行显示固定数量字节，默认 16
- 针对中等规模数据，首版优先实现清晰的数据映射与交互逻辑
- 如 `gpui` 容器能力允许，列表区域采用可见区域渲染思路，避免一次性构建全部行视图

## 对外 API

### 构造与配置

建议首版暴露：

- `HexEditor::new(buffer, config)`
- `HexEditorConfig`
  - `bytes_per_row`
  - `show_ascii`
  - `show_toolbar`
  - `show_status_bar`
  - `read_only`

### 事件

- `on_change`
- `on_selection_change`
- `on_cursor_change`

### 状态读取

提供轻量状态读取对象，例如 `HexEditorStateHandle`，便于宿主组件获取：

- 当前偏移
- 当前选区
- 是否脏数据
- 当前编辑模式

## 交互设计

### 键盘

- 方向键：逐字节/逐行移动
- `Home` / `End`：行首/行尾
- `PageUp` / `PageDown`：按页移动
- Hex 区支持半字节输入
- ASCII 区支持可显示字符直接写入

### 鼠标

- 单击定位光标
- 拖拽建立连续选区
- Hover 提示当前单元格焦点感

### 编辑行为

- Hex 区输入单个十六进制字符时，进入半字节编辑流程
- 两次合法输入后写入完整字节
- ASCII 区输入可见字符时，直接更新对应字节
- 只读模式下禁用所有写入行为

## 视觉设计

### 风格

- 现代简洁风
- 保持面板层级，但避免过重装饰
- 强调可读性、对齐与轻量交互反馈

### Theme 集成

优先使用 `gpui_component` theme 的语义色：

- 背景层级
- 文本主次色
- 边框色
- 主色 / focus ring
- hover / active 状态色

仅在必要时补少量 HexEditor 专用派生 token，例如：

- 当前字节高亮背景
- 当前行弱高亮背景
- 选区高亮背景

## 测试策略

首版以状态逻辑测试为主。

### 核心测试点

- 偏移与行列换算
- 光标移动边界
- 选区扩展与收缩
- Hex 输入写入逻辑
- ASCII 输入写入逻辑
- 只读模式禁止修改

## 实施顺序

1. 建立 crate 与依赖
2. 定义配置、状态与样式结构
3. 实现缓冲区映射与编辑逻辑
4. 实现基础布局与主题接入
5. 实现键鼠交互
6. 实现工具栏与状态栏
7. 补充测试与示例
