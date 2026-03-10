# Compact Layout And File Editor Example Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 收紧 HexEditor 布局密度，并新增一个可打开和保存本地文件的示例应用。

**Architecture:** 保持现有状态模型不变，新增少量只读/导出接口支撑示例读写；视图层通过调小尺寸参数实现更紧凑外观；示例层增加文件载入与保存逻辑。

**Tech Stack:** Rust、gpui、gpui_component、std::fs、cargo test、cargo check

---
