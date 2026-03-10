# Direct Editing Interaction Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 `HexEditor` 成为真正可聚焦、可点击、可键盘直接编辑字节的组件。

**Architecture:** 继续以状态层为核心，先通过测试补齐连续输入、删除与模式切换，再将 `gpui` 的焦点、鼠标与键盘事件接入到组件视图。示例应用直接复用组件事件能力进行本地文件编辑和保存。

**Tech Stack:** Rust、gpui、gpui_component、cargo test、cargo check

---
