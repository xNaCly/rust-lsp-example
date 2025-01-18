#[derive(Debug, Default)]
pub struct LspError {
    line: usize,
    start: usize,
    end: usize,
    message: String,
}

impl LspError {
    pub fn new(line: usize, start: usize, end: usize, message: String) -> Self {
        Self {
            line,
            start,
            end,
            message,
        }
    }
}
