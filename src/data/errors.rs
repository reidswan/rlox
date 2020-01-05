use std::io;

#[derive(Debug)]
pub struct ErrorData {
    pub message: String,
    pub line_no: usize,
    pub location: String
}

#[derive(Debug)]
pub enum LoxError {
    IoError(io::Error),
    ScannerError(ErrorData),
}

