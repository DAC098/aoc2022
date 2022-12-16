use std::io::{Lines, BufReader, Read, BufRead};

pub struct LineReader<T> {
    count: usize,
    lines: Lines<BufReader<T>>
}

impl<T: Read> LineReader<T> {
    pub fn new(inner: BufReader<T>) -> Self {
        LineReader { count: 0, lines: inner.lines() }
    }

    pub fn next_line(&mut self) -> std::io::Result<Option<String>> {
        self.count += 1;

        let Some(result) = self.lines.next() else {
            return Ok(None);
        };

        Ok(Some(result?))
    }

    pub fn get_count(&self) -> &usize {
        &self.count
    }
} 