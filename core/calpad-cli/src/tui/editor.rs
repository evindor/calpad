pub struct Editor {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub viewport_height: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            viewport_height: 24,
        }
    }

    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn set_content(&mut self, content: &str) {
        self.lines = content.split('\n').map(String::from).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }

    fn line_char_len(&self) -> usize {
        self.lines[self.cursor_row].chars().count()
    }

    fn char_to_byte(&self, row: usize, char_offset: usize) -> usize {
        self.lines[row]
            .char_indices()
            .nth(char_offset)
            .map(|(i, _)| i)
            .unwrap_or(self.lines[row].len())
    }

    pub fn insert_char(&mut self, c: char) {
        let byte_pos = self.char_to_byte(self.cursor_row, self.cursor_col);
        self.lines[self.cursor_row].insert(byte_pos, c);
        self.cursor_col += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let byte_start = self.char_to_byte(self.cursor_row, self.cursor_col - 1);
            let byte_end = self.char_to_byte(self.cursor_row, self.cursor_col);
            self.lines[self.cursor_row].replace_range(byte_start..byte_end, "");
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.line_char_len();
            self.lines[self.cursor_row].push_str(&current_line);
            self.ensure_visible();
        }
    }

    pub fn delete(&mut self) {
        let len = self.line_char_len();
        if self.cursor_col < len {
            let byte_start = self.char_to_byte(self.cursor_row, self.cursor_col);
            let byte_end = self.char_to_byte(self.cursor_row, self.cursor_col + 1);
            self.lines[self.cursor_row].replace_range(byte_start..byte_end, "");
        } else if self.cursor_row < self.lines.len() - 1 {
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next_line);
        }
    }

    pub fn newline(&mut self) {
        let byte_pos = self.char_to_byte(self.cursor_row, self.cursor_col);
        let rest = self.lines[self.cursor_row][byte_pos..].to_string();
        self.lines[self.cursor_row].truncate(byte_pos);
        self.cursor_row += 1;
        self.lines.insert(self.cursor_row, rest);
        self.cursor_col = 0;
        self.ensure_visible();
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.line_char_len();
            self.ensure_visible();
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor_col < self.line_char_len() {
            self.cursor_col += 1;
        } else if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            self.cursor_col = 0;
            self.ensure_visible();
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cursor_col.min(self.line_char_len());
            self.ensure_visible();
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_row < self.lines.len() - 1 {
            self.cursor_row += 1;
            self.cursor_col = self.cursor_col.min(self.line_char_len());
            self.ensure_visible();
        }
    }

    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor_col = self.line_char_len();
    }

    pub fn page_up(&mut self) {
        let jump = self.viewport_height.saturating_sub(2).max(1);
        self.cursor_row = self.cursor_row.saturating_sub(jump);
        self.cursor_col = self.cursor_col.min(self.line_char_len());
        self.ensure_visible();
    }

    pub fn page_down(&mut self) {
        let jump = self.viewport_height.saturating_sub(2).max(1);
        self.cursor_row = (self.cursor_row + jump).min(self.lines.len() - 1);
        self.cursor_col = self.cursor_col.min(self.line_char_len());
        self.ensure_visible();
    }

    pub fn ensure_visible(&mut self) {
        if self.cursor_row < self.scroll_offset {
            self.scroll_offset = self.cursor_row;
        }
        if self.cursor_row >= self.scroll_offset + self.viewport_height {
            self.scroll_offset = self.cursor_row - self.viewport_height + 1;
        }
    }
}
