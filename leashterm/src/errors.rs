//! Structured errors. Every error carries a machine-readable `kind`, an optional
//! line number, a message and a concrete hint, so a model can feed the JSON back
//! to itself and fix its own program.

#[derive(Debug, Clone)]
pub struct LangError {
    pub kind: &'static str,
    pub line: Option<usize>,
    pub message: String,
    pub hint: String,
}

impl LangError {
    pub fn new(
        kind: &'static str,
        line: Option<usize>,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        LangError {
            kind,
            line,
            message: message.into(),
            hint: hint.into(),
        }
    }

    pub fn to_json(&self) -> String {
        let line = match self.line {
            Some(l) => l.to_string(),
            None => "null".to_string(),
        };
        format!(
            "{{\"error\":\"{}\",\"line\":{},\"message\":\"{}\",\"hint\":\"{}\"}}",
            self.kind,
            line,
            escape(&self.message),
            escape(&self.hint)
        )
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
