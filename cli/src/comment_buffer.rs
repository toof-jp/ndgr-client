use std::collections::VecDeque;

use unicode_width::UnicodeWidthChar;

pub struct CommentBuffer {
    comments: VecDeque<String>,
    width: usize,
    height: usize,
}

impl CommentBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            comments: VecDeque::new(),
            width,
            height,
        }
    }

    pub fn push(&mut self, comment: String) {
        let mut current_line = String::new();
        let mut current_width = 0;

        for c in comment.chars() {
            let c_width = c.width_cjk().unwrap_or(0);

            if current_width + c_width > self.width {
                self.comments.push_back(current_line.clone());
                current_line.clear();
                current_width = 0;
            }

            current_line.push(c);
            current_width += c_width;
        }

        if !current_line.is_empty() {
            self.comments.push_back(current_line);
        }

        while self.comments.len() > self.height {
            self.comments.pop_front();
        }
    }

    pub fn comments(&self) -> &VecDeque<String> {
        &self.comments
    }
}
