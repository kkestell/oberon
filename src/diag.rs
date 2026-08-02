#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos {
    pub line: u32, // 1-based
    pub col: u32,  // 1-based
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub pos: Pos,
    pub msg: String,
}

impl Diagnostic {
    pub fn new(pos: Pos, msg: impl Into<String>) -> Self {
        Diagnostic {
            pos,
            msg: msg.into(),
        }
    }
}
