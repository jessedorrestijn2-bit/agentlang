//! Interpreter. Every side effect (read, write) must be covered by a `needs`
//! permission, and every action is written to a hash-chained audit log.
//!
//! NOTE: the log uses std's DefaultHasher as a placeholder. It detects accidental
//! edits but is NOT cryptographically secure. Replace it with SHA-256 later.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::errors::LangError;
use crate::parser::{Expr, Stmt, StmtKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Num(f64),
    Bool(bool),
    Nothing,
}

impl Value {
    fn show(&self) -> String {
        match self {
            Value::Str(s) => s.clone(),
            Value::Num(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::Bool(b) => b.to_string(),
            Value::Nothing => "nothing".to_string(),
        }
    }

    fn truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub seq: usize,
    pub action: String,
    pub target: String,
    pub ok: bool,
    pub prev: u64,
    pub hash: u64,
}

fn entry_hash(seq: usize, action: &str, target: &str, ok: bool, prev: u64) -> u64 {
    let mut h = DefaultHasher::new();
    (seq, action, target, ok, prev).hash(&mut h);
    h.finish()
}

/// Recomputes the whole chain; returns false if any entry was changed,
/// removed or reordered.
pub fn chain_is_valid(log: &[LogEntry]) -> bool {
    let mut prev = 0u64;
    for (i, e) in log.iter().enumerate() {
        if e.seq != i
            || e.prev != prev
            || e.hash != entry_hash(e.seq, &e.action, &e.target, e.ok, e.prev)
        {
            return false;
        }
        prev = e.hash;
    }
    true
}

pub struct Interp {
    caps: Vec<(String, String)>,
    vars: HashMap<String, Value>,
    pub output: Vec<String>,
    pub log: Vec<LogEntry>,
}

impl Interp {
    pub fn new() -> Self {
        Interp {
            caps: Vec::new(),
            vars: HashMap::new(),
            output: Vec::new(),
            log: Vec::new(),
        }
    }

    pub fn run(&mut self, prog: &[Stmt]) -> Result<(), LangError> {
        for stmt in prog {
            self.exec(stmt)?;
        }
        Ok(())
    }

    fn record(&mut self, action: &str, target: &str, ok: bool) {
        let prev = self.log.last().map(|e| e.hash).unwrap_or(0);
        let seq = self.log.len();
        let hash = entry_hash(seq, action, target, ok, prev);
        self.log.push(LogEntry {
            seq,
            action: action.to_string(),
            target: target.to_string(),
            ok,
            prev,
            hash,
        });
    }

    fn exec(&mut self, stmt: &Stmt) -> Result<(), LangError> {
        match &stmt.kind {
            StmtKind::Needs { action, target } => {
                self.caps.push((action.clone(), target.clone()));
                self.record("needs", &format!("{}({})", action, target), true);
                Ok(())
            }
            StmtKind::Let(name, expr) => {
                let value = self.eval(expr, stmt.line)?;
                self.vars.insert(name.clone(), value);
                Ok(())
            }
            StmtKind::Do(expr) => {
                self.eval(expr, stmt.line)?;
                Ok(())
            }
            StmtKind::Verify(expr) => {
                let value = self.eval(expr, stmt.line)?;
                if value.truthy() {
                    self.record("verify", "passed", true);
                    Ok(())
                } else {
                    self.record("verify", "failed", false);
                    Err(LangError::new(
                        "verify_failed",
                        Some(stmt.line),
                        "a verify check did not hold, so the program was stopped",
                        "check the values that are compared, or handle the mismatch before this line",
                    ))
                }
            }
            StmtKind::Retry(times, body) => self.retry(*times, body, stmt.line),
        }
    }

    fn retry(&mut self, times: u32, body: &[Stmt], line: usize) -> Result<(), LangError> {
        let mut last_message = String::new();
        for attempt in 1..=times {
            let mut failure: Option<LangError> = None;
            for stmt in body {
                if let Err(e) = self.exec(stmt) {
                    failure = Some(e);
                    break;
                }
            }
            match failure {
                None => return Ok(()),
                Some(e) => {
                    // A missing permission will not fix itself, so never retry it.
                    if e.kind == "capability_denied" {
                        return Err(e);
                    }
                    self.record("retry", &format!("attempt {} of {} failed", attempt, times), false);
                    last_message = e.message;
                }
            }
        }
        Err(LangError::new(
            "retries_exhausted",
            Some(line),
            format!("the block failed all {} attempts; last error: {}", times, last_message),
            "fix the cause, or handle the failure outside the retry block",
        ))
    }

    fn eval(&mut self, expr: &Expr, line: usize) -> Result<Value, LangError> {
        match expr {
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Num(n) => Ok(Value::Num(*n)),
            Expr::Var(name) => self.vars.get(name).cloned().ok_or_else(|| {
                LangError::new(
                    "unknown_variable",
                    Some(line),
                    format!("variable '{}' has not been defined", name),
                    format!("define it first with: let {} = ...", name),
                )
            }),
            Expr::Eq(a, b) => {
                let left = self.eval(a, line)?;
                let right = self.eval(b, line)?;
                Ok(Value::Bool(left == right))
            }
            Expr::Call(name, args) => {
                let mut values = Vec::new();
                for arg in args {
                    values.push(self.eval(arg, line)?);
                }
                self.call(name, values, line)
            }
        }
    }

    fn require(&mut self, action: &str, target: &str, line: usize) -> Result<(), LangError> {
        let allowed = self.caps.iter().any(|(a, t)| a == action && t == target);
        if allowed {
            return Ok(());
        }
        self.record(action, target, false);
        Err(LangError::new(
            "capability_denied",
            Some(line),
            format!("{}(\"{}\") is not permitted", action, target),
            format!("add this line at the top of the program: needs {}(\"{}\")", action, target),
        ))
    }

    fn call(&mut self, name: &str, args: Vec<Value>, line: usize) -> Result<Value, LangError> {
        match (name, args.as_slice()) {
            ("print", [value]) => {
                self.output.push(value.show());
                Ok(Value::Nothing)
            }
            ("len", [Value::Str(text)]) => Ok(Value::Num(text.chars().count() as f64)),
            ("read", [Value::Str(path)]) => {
                self.require("read", path, line)?;
                match std::fs::read_to_string(path) {
                    Ok(text) => {
                        self.record("read", path, true);
                        Ok(Value::Str(text))
                    }
                    Err(e) => {
                        self.record("read", path, false);
                        Err(LangError::new(
                            "io_error",
                            Some(line),
                            format!("could not read '{}': {}", path, e),
                            "check that the file exists and can be opened",
                        ))
                    }
                }
            }
            ("write", [Value::Str(path), Value::Str(text)]) => {
                self.require("write", path, line)?;
                match std::fs::write(path, text) {
                    Ok(()) => {
                        self.record("write", path, true);
                        Ok(Value::Nothing)
                    }
                    Err(e) => {
                        self.record("write", path, false);
                        Err(LangError::new(
                            "io_error",
                            Some(line),
                            format!("could not write '{}': {}", path, e),
                            "check that the folder exists and is writable",
                        ))
                    }
                }
            }
            _ => Err(LangError::new(
                "bad_call",
                Some(line),
                format!("unknown function or wrong arguments: {}(...) with {} argument(s)", name, args.len()),
                "builtins: print(x), len(text), read(path), write(path, text)",
            )),
        }
    }
}
