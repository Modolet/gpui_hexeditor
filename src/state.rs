use std::ops::RangeInclusive;

pub type SelectionRange = RangeInclusive<usize>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditError {
    ReadOnly,
    InvalidHexDigit,
    NonPrintableAscii,
    OutOfBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivePane {
    #[default]
    Hex,
    Ascii,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexEditorState {
    bytes: Vec<u8>,
    bytes_per_row: usize,
    cursor_offset: usize,
    selection: Option<SelectionRange>,
    selection_anchor: Option<usize>,
    active_pane: ActivePane,
    read_only: bool,
    dirty: bool,
    pending_nibble: Option<u8>,
}

impl HexEditorState {
    pub fn new(bytes: Vec<u8>, bytes_per_row: usize) -> Self {
        let has_bytes = !bytes.is_empty();
        Self {
            bytes,
            bytes_per_row: bytes_per_row.max(1),
            cursor_offset: 0,
            selection: None,
            selection_anchor: None,
            active_pane: ActivePane::Hex,
            read_only: false,
            dirty: false,
            pending_nibble: None,
        }
        .with_valid_cursor(has_bytes)
    }

    fn with_valid_cursor(mut self, has_bytes: bool) -> Self {
        if !has_bytes {
            self.cursor_offset = 0;
        }
        self
    }

    fn max_offset(&self) -> usize {
        self.bytes.len().saturating_sub(1)
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn bytes_per_row(&self) -> usize {
        self.bytes_per_row
    }

    pub fn row_count(&self) -> usize {
        self.len().div_ceil(self.bytes_per_row)
    }

    pub fn cursor_offset(&self) -> usize {
        self.cursor_offset
    }

    pub fn active_pane(&self) -> ActivePane {
        self.active_pane
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn set_active_pane(&mut self, pane: ActivePane) {
        self.active_pane = pane;
        self.pending_nibble = None;
    }

    pub fn set_bytes_per_row(&mut self, bytes_per_row: usize) {
        self.bytes_per_row = bytes_per_row.max(1);
        self.cursor_offset = self.cursor_offset.min(self.max_offset());
    }

    pub fn offset_to_row_col(&self, offset: usize) -> (usize, usize) {
        if self.is_empty() {
            return (0, 0);
        }
        let clamped = offset.min(self.max_offset());
        (clamped / self.bytes_per_row, clamped % self.bytes_per_row)
    }

    pub fn row_col_to_offset(&self, row: usize, col: usize) -> usize {
        if self.is_empty() {
            return 0;
        }
        (row * self.bytes_per_row + col.min(self.bytes_per_row.saturating_sub(1)))
            .min(self.max_offset())
    }

    pub fn row_bytes(&self, row: usize) -> &[u8] {
        let start = row.saturating_mul(self.bytes_per_row).min(self.bytes.len());
        let end = (start + self.bytes_per_row).min(self.bytes.len());
        &self.bytes[start..end]
    }

    pub fn set_cursor(&mut self, offset: usize) {
        self.cursor_offset = offset.min(self.max_offset());
        self.selection = None;
        self.selection_anchor = None;
        self.pending_nibble = None;
    }

    pub fn move_left(&mut self) {
        self.cursor_offset = self.cursor_offset.saturating_sub(1);
        self.pending_nibble = None;
    }

    pub fn move_right(&mut self) {
        if !self.is_empty() {
            self.cursor_offset = (self.cursor_offset + 1).min(self.max_offset());
        }
        self.pending_nibble = None;
    }

    pub fn move_up(&mut self) {
        self.cursor_offset = self.cursor_offset.saturating_sub(self.bytes_per_row);
        self.pending_nibble = None;
    }

    pub fn move_down(&mut self) {
        if !self.is_empty() {
            self.cursor_offset = (self.cursor_offset + self.bytes_per_row).min(self.max_offset());
        }
        self.pending_nibble = None;
    }

    pub fn move_home(&mut self) {
        let (row, _) = self.offset_to_row_col(self.cursor_offset);
        self.cursor_offset = self.row_col_to_offset(row, 0);
        self.pending_nibble = None;
    }

    pub fn move_end(&mut self) {
        let (row, _) = self.offset_to_row_col(self.cursor_offset);
        self.cursor_offset = self.row_col_to_offset(row, self.bytes_per_row.saturating_sub(1));
        self.pending_nibble = None;
    }

    pub fn move_page_up(&mut self, rows: usize) {
        self.cursor_offset = self
            .cursor_offset
            .saturating_sub(self.bytes_per_row * rows.max(1));
        self.pending_nibble = None;
    }

    pub fn move_page_down(&mut self, rows: usize) {
        if !self.is_empty() {
            self.cursor_offset =
                (self.cursor_offset + self.bytes_per_row * rows.max(1)).min(self.max_offset());
        }
        self.pending_nibble = None;
    }

    pub fn begin_selection(&mut self, anchor: usize) {
        let clamped = anchor.min(self.max_offset());
        self.selection_anchor = Some(clamped);
        self.cursor_offset = clamped;
        self.selection = Some(clamped..=clamped);
        self.pending_nibble = None;
    }

    pub fn update_selection(&mut self, focus: usize) {
        let focus = focus.min(self.max_offset());
        let anchor = self.selection_anchor.unwrap_or(self.cursor_offset);
        let start = anchor.min(focus);
        let end = anchor.max(focus);
        self.cursor_offset = focus;
        self.selection = Some(start..=end);
        self.pending_nibble = None;
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.selection_anchor = None;
    }

    pub fn selection_anchor(&self) -> Option<usize> {
        self.selection_anchor
    }

    pub fn selected_range(&self) -> Option<&SelectionRange> {
        self.selection.as_ref()
    }

    pub fn selection_len(&self) -> usize {
        self.selection
            .as_ref()
            .map(|range| range.end() - range.start() + 1)
            .unwrap_or(0)
    }

    pub fn is_selected(&self, offset: usize) -> bool {
        self.selection
            .as_ref()
            .map(|range| range.contains(&offset))
            .unwrap_or(false)
    }

    pub fn selected_hex_string(&self) -> Option<String> {
        let range = self.selection.as_ref()?;
        Some(
            self.bytes[*range.start()..=*range.end()]
                .iter()
                .map(|byte| format!("{byte:02X}"))
                .collect::<Vec<_>>()
                .join(" "),
        )
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
        if read_only {
            self.pending_nibble = None;
        }
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn pending_nibble(&self) -> Option<u8> {
        self.pending_nibble
    }

    pub fn byte_at_cursor(&self) -> u8 {
        self.bytes
            .get(self.cursor_offset)
            .copied()
            .unwrap_or_default()
    }

    pub fn bytes_cloned(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    pub fn overwrite_byte(&mut self, value: u8) -> Result<(), EditError> {
        if self.read_only {
            return Err(EditError::ReadOnly);
        }
        if self.is_empty() {
            return Err(EditError::OutOfBounds);
        }

        self.bytes[self.cursor_offset] = value;
        self.pending_nibble = None;
        self.dirty = true;
        Ok(())
    }

    pub fn delete(&mut self) -> Result<(), EditError> {
        self.overwrite_byte(0x00)
    }

    pub fn backspace(&mut self) -> Result<(), EditError> {
        if self.read_only {
            return Err(EditError::ReadOnly);
        }
        if self.is_empty() {
            return Err(EditError::OutOfBounds);
        }

        self.move_left();
        self.overwrite_byte(0x00)
    }

    pub fn handle_text_input(&mut self, text: &str) -> bool {
        let Some(character) = text.chars().next() else {
            return false;
        };

        match self.active_pane {
            ActivePane::Hex => self.apply_hex_digit(character).is_ok(),
            ActivePane::Ascii => self.apply_ascii_char(character).is_ok(),
        }
    }

    pub fn handle_key(&mut self, key: &str, page_rows: usize) -> bool {
        match key {
            "left" => self.move_left(),
            "right" => self.move_right(),
            "up" => self.move_up(),
            "down" => self.move_down(),
            "home" => self.move_home(),
            "end" => self.move_end(),
            "pageup" => self.move_page_up(page_rows),
            "pagedown" => self.move_page_down(page_rows),
            "backspace" => return self.backspace().is_ok(),
            "delete" => return self.delete().is_ok(),
            _ => return false,
        }

        true
    }

    pub fn apply_hex_digit(&mut self, digit: char) -> Result<(), EditError> {
        if self.read_only {
            return Err(EditError::ReadOnly);
        }
        if self.is_empty() {
            return Ok(());
        }

        let value = digit.to_digit(16).ok_or(EditError::InvalidHexDigit)? as u8;
        if let Some(high) = self.pending_nibble.take() {
            self.bytes[self.cursor_offset] = (high << 4) | value;
            self.dirty = true;
            self.move_right();
        } else {
            self.pending_nibble = Some(value);
        }
        Ok(())
    }

    pub fn apply_ascii_char(&mut self, character: char) -> Result<(), EditError> {
        if self.read_only {
            return Err(EditError::ReadOnly);
        }
        if self.is_empty() {
            return Ok(());
        }
        if !character.is_ascii_graphic() && character != ' ' {
            return Err(EditError::NonPrintableAscii);
        }

        self.bytes[self.cursor_offset] = character as u8;
        self.pending_nibble = None;
        self.dirty = true;
        self.move_right();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ActivePane, HexEditorState};

    #[test]
    fn maps_offset_to_row_and_column() {
        let state = HexEditorState::new(vec![0; 64], 16);
        assert_eq!(state.offset_to_row_col(19), (1, 3));
    }

    #[test]
    fn maps_row_and_column_back_to_offset() {
        let state = HexEditorState::new(vec![0; 64], 16);
        assert_eq!(state.row_col_to_offset(2, 5), 37);
    }

    #[test]
    fn moves_cursor_horizontally() {
        let mut state = HexEditorState::new(vec![0; 64], 16);
        state.move_right();
        state.move_right();
        state.move_left();
        assert_eq!(state.cursor_offset(), 1);
    }

    #[test]
    fn moves_cursor_vertically() {
        let mut state = HexEditorState::new(vec![0; 64], 16);
        state.set_cursor(18);
        state.move_down();
        state.move_up();
        assert_eq!(state.cursor_offset(), 18);
    }

    #[test]
    fn moves_to_row_bounds() {
        let mut state = HexEditorState::new(vec![0; 64], 16);
        state.set_cursor(22);
        state.move_home();
        assert_eq!(state.cursor_offset(), 16);
        state.move_end();
        assert_eq!(state.cursor_offset(), 31);
    }

    #[test]
    fn never_moves_past_file_bounds() {
        let mut state = HexEditorState::new(vec![0; 10], 8);
        state.move_left();
        assert_eq!(state.cursor_offset(), 0);
        state.set_cursor(9);
        state.move_right();
        state.move_down();
        assert_eq!(state.cursor_offset(), 9);
    }

    #[test]
    fn setting_cursor_clears_previous_selection() {
        let mut state = HexEditorState::new(vec![0; 64], 16);
        state.begin_selection(4);
        state.update_selection(12);

        state.set_cursor(20);

        assert_eq!(state.selected_range(), None);
        assert_eq!(state.cursor_offset(), 20);
    }

    #[test]
    fn drag_selection_expands_forward() {
        let mut state = HexEditorState::new(vec![0; 64], 16);

        state.begin_selection(4);
        state.update_selection(12);

        assert_eq!(state.selected_range(), Some(&(4..=12)));
        assert_eq!(state.selection_len(), 9);
    }

    #[test]
    fn drag_selection_normalizes_reverse_range() {
        let mut state = HexEditorState::new(vec![0; 64], 16);

        state.begin_selection(18);
        state.update_selection(3);

        assert_eq!(state.selected_range(), Some(&(3..=18)));
    }

    #[test]
    fn selection_len_counts_across_rows() {
        let mut state = HexEditorState::new(vec![0; 64], 16);

        state.begin_selection(14);
        state.update_selection(19);

        assert_eq!(state.selection_len(), 6);
    }

    #[test]
    fn hex_edit_keeps_pending_nibble_after_first_digit() {
        let mut state = HexEditorState::new(vec![0xAB], 16);

        state.apply_hex_digit('C').unwrap();

        assert_eq!(state.pending_nibble(), Some(0xC));
        assert_eq!(state.byte_at_cursor(), 0xAB);
    }

    #[test]
    fn hex_edit_writes_byte_after_second_digit() {
        let mut state = HexEditorState::new(vec![0xAB, 0x00], 16);

        state.apply_hex_digit('C').unwrap();
        state.apply_hex_digit('D').unwrap();

        assert_eq!(state.bytes()[0], 0xCD);
        assert_eq!(state.pending_nibble(), None);
        assert_eq!(state.cursor_offset(), 1);
    }

    #[test]
    fn ascii_edit_writes_printable_character() {
        let mut state = HexEditorState::new(vec![0x00, 0x00], 16);

        state.apply_ascii_char('Z').unwrap();

        assert_eq!(state.bytes()[0], b'Z');
        assert_eq!(state.cursor_offset(), 1);
    }

    #[test]
    fn read_only_state_rejects_edits() {
        let mut state = HexEditorState::new(vec![0x00], 16);
        state.set_read_only(true);

        assert!(state.apply_ascii_char('A').is_err());
        assert!(state.apply_hex_digit('F').is_err());
        assert_eq!(state.byte_at_cursor(), 0x00);
    }

    #[test]
    fn edit_marks_buffer_as_dirty() {
        let mut state = HexEditorState::new(vec![0x00, 0x00], 16);

        state.apply_ascii_char('A').unwrap();

        assert!(state.is_dirty());
    }

    #[test]
    fn overwrite_byte_updates_current_offset() {
        let mut state = HexEditorState::new(vec![0x00, 0x11], 16);
        state.set_cursor(1);

        state.overwrite_byte(0xFE).unwrap();

        assert_eq!(state.bytes()[1], 0xFE);
        assert!(state.is_dirty());
    }

    #[test]
    fn bytes_cloned_returns_current_buffer() {
        let state = HexEditorState::new(vec![0xDE, 0xAD, 0xBE, 0xEF], 16);
        assert_eq!(state.bytes_cloned(), vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn active_pane_switches() {
        let mut state = HexEditorState::new(vec![0; 4], 16);
        state.set_active_pane(ActivePane::Ascii);
        assert_eq!(state.active_pane(), ActivePane::Ascii);
    }

    #[test]
    fn backspace_clears_previous_byte_and_moves_cursor() {
        let mut state = HexEditorState::new(vec![0xAA, 0xBB, 0xCC], 16);
        state.set_cursor(2);

        state.backspace().unwrap();

        assert_eq!(state.cursor_offset(), 1);
        assert_eq!(state.bytes()[1], 0x00);
    }

    #[test]
    fn delete_clears_current_byte() {
        let mut state = HexEditorState::new(vec![0xAA, 0xBB], 16);
        state.set_cursor(1);

        state.delete().unwrap();

        assert_eq!(state.bytes()[1], 0x00);
        assert_eq!(state.cursor_offset(), 1);
    }

    #[test]
    fn handle_text_input_uses_active_pane() {
        let mut state = HexEditorState::new(vec![0x00, 0x00], 16);
        state.set_active_pane(ActivePane::Ascii);

        assert!(state.handle_text_input("A"));
        assert_eq!(state.bytes()[0], b'A');

        state.set_active_pane(ActivePane::Hex);
        state.set_cursor(1);
        assert!(state.handle_text_input("C"));
        assert!(state.handle_text_input("D"));
        assert_eq!(state.bytes()[1], 0xCD);
    }

    #[test]
    fn handle_navigation_keys_moves_cursor() {
        let mut state = HexEditorState::new(vec![0; 64], 16);
        state.set_cursor(17);

        assert!(state.handle_key("down", 16));
        assert_eq!(state.cursor_offset(), 33);
        assert!(state.handle_key("pageup", 2));
        assert_eq!(state.cursor_offset(), 1);
        assert!(state.handle_key("end", 16));
        assert_eq!(state.cursor_offset(), 15);
    }

    #[test]
    fn selected_hex_string_formats_range() {
        let mut state = HexEditorState::new(vec![0xDE, 0xAD, 0xBE, 0xEF], 16);
        state.begin_selection(1);
        state.update_selection(3);

        assert_eq!(state.selected_hex_string().as_deref(), Some("AD BE EF"));
    }

    #[test]
    fn selection_anchor_tracks_drag_start() {
        let mut state = HexEditorState::new(vec![0; 16], 16);
        state.begin_selection(5);
        state.update_selection(9);

        assert_eq!(state.selection_anchor(), Some(5));
        assert_eq!(state.selected_range(), Some(&(5..=9)));
    }
}
