# Internal Scroll And Compact Layout Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 HexEditor 内容区支持组件内部滚动，并进一步收紧整体布局密度。

**Architecture:** 保持工具栏和状态栏固定，仅将字节内容区改为内部滚动容器；通过降低行高、单元格宽度、边距和区块间距，使视觉更紧凑；必要时让内容区高度由配置控制。

**Tech Stack:** Rust、gpui、gpui_component、cargo test、cargo check

---
