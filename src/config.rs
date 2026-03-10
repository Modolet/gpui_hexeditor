/// `HexEditor` 的外部配置项。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexEditorConfig {
    pub bytes_per_row: usize,
    pub show_ascii: bool,
    pub show_toolbar: bool,
    pub show_status_bar: bool,
    pub read_only: bool,
    pub visible_rows: usize,
}

impl HexEditorConfig {
    /// 设置每行显示的字节数。
    pub fn with_bytes_per_row(mut self, bytes_per_row: usize) -> Self {
        self.bytes_per_row = bytes_per_row.max(1);
        self
    }

    /// 设置是否只读。
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// 设置内容区可见的逻辑行数。
    pub fn with_visible_rows(mut self, visible_rows: usize) -> Self {
        self.visible_rows = visible_rows.max(1);
        self
    }
}

impl Default for HexEditorConfig {
    fn default() -> Self {
        Self {
            bytes_per_row: 16,
            show_ascii: true,
            show_toolbar: true,
            show_status_bar: true,
            read_only: false,
            visible_rows: 20,
        }
    }
}
