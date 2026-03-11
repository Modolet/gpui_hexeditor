# Publish gpui_hexeditor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prepare `gpui_hexeditor` for GitHub and crates.io publication, then merge dependent worktrees and switch `siois_rs` to the published crate version.

**Architecture:** First complete package metadata, license, README, and crate documentation in `gpui_hexeditor`, then validate with `cargo test` and `cargo publish --dry-run`. After successful publication, merge both feature worktrees back to their default branches, switch `siois_rs` to the crates.io dependency, verify again, and remove the temporary worktrees.

**Tech Stack:** Rust, Cargo, Git, crates.io, GitHub SSH remote

---

### Task 1: Prepare `gpui_hexeditor` package metadata

**Files:**
- Modify: `Cargo.toml`
- Modify: `README.md`
- Modify: `src/lib.rs`
- Modify: `src/widget.rs`
- Create: `LICENSE`
- Create: `docs/plans/2026-03-11-publish-gpui-hexeditor.md`

**Step 1: Add missing publish metadata**
- Add `description`, `license`, `repository`, `homepage`, `documentation`, `readme`, `keywords`, and `categories`.

**Step 2: Add crate documentation**
- Add crate-level docs to `src/lib.rs`.
- Add missing doc comments to key public widget APIs.

**Step 3: Add license and improve README**
- Add MIT `LICENSE`.
- Rewrite README for crates.io consumers.

**Step 4: Verify package quality**
Run: `cargo fmt && cargo test -q && cargo publish --dry-run`
Expected: all tests pass and dry-run succeeds.

**Step 5: Commit**
```bash
git add Cargo.toml README.md src/lib.rs src/widget.rs LICENSE docs/plans/2026-03-11-publish-gpui-hexeditor.md
git commit -m "feat(crate): 补全 gpui_hexeditor 发布元数据"
```

### Task 2: Publish and push `gpui_hexeditor`

**Files:**
- Modify: git remote configuration only if needed

**Step 1: Ensure remote is configured**
Run: `git remote -v`
Expected: `origin` points to `git@github.com:Modolet/gpui_hexeditor.git`

**Step 2: Push default branch**
Run: `git push -u origin master`
Expected: push succeeds.

**Step 3: Publish crate**
Run: `cargo publish`
Expected: publication succeeds and crate version becomes available on crates.io.

### Task 3: Merge worktrees and switch dependency in `siois_rs`

**Files:**
- Modify: `/home/modolet/workspace/siois_rs/Cargo.toml`
- Modify: `/home/modolet/workspace/siois_rs/Cargo.lock`

**Step 1: Merge `gpui_hexeditor` feature branch to default branch**
- Merge `feat/hexeditor-hide-ascii` if it still contains unique commits.

**Step 2: Merge `siois_rs` feature branch to default branch**
- Merge `feat/sis7500-mtp-hexeditor` into `master`.

**Step 3: Replace path dependency with crates.io version**
- Switch `gpui_hexeditor = { path = ... }` to `gpui_hexeditor = "0.1.0"`.

**Step 4: Verify dependent repo**
Run: `cargo fmt && cargo test sis7500_mtp -- --nocapture && cargo check -q`
Expected: all checks pass.

**Step 5: Commit**
```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(cargo): 切换 gpui_hexeditor 到 crates.io"
```

### Task 4: Cleanup worktrees

**Files:**
- No tracked files changed

**Step 1: Remove the worktrees**
- Remove `.worktrees/hexeditor-hide-ascii`
- Remove `.worktrees/sis7500-mtp-hexeditor`

**Step 2: Verify cleanup**
Run: `git worktree list`
Expected: feature worktrees are gone.
