use std::time::Instant;

#[derive(Debug)]
pub struct StatusMessage {
    pub text: String,
    pub level: MessageLevel,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum MessageLevel {
    Info,
    Success,
    Warning,
    Error,
}

