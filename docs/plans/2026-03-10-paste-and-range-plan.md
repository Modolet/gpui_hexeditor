# Paste And Range Feedback Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 HexEditor 增加十六进制粘贴、Shift 点击扩选和状态栏选区范围显示。

**Architecture:** 先在状态层补粘贴解析与范围格式化测试，再把 `gpui` 剪贴板读取和 Shift 点击事件接到组件，最后更新状态栏与文档。

**Tech Stack:** Rust、gpui、gpui_component、cargo test、cargo check

---
