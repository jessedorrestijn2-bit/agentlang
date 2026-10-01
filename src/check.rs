//! Static check: runs after parsing and BEFORE anything executes.
//!
//! It collects the declared `needs` permissions and rejects the whole program if
//! a `read(...)` or `write(...)` is not covered. It checks:
//!   - literal paths and URLs:   read("a.txt"), fetch("https://example.com/x")
//!   - loop variables over a literal list of texts:
//!         for p in ["a.txt", "b.txt"] { read(p) }   (checks both files)
//!
//! Paths that are only known while running (for example the result of another
//! call) are still checked by the interpreter at runtime.

use std::collections::HashMap;

use crate::errors::LangError;
use crate::net;
use crate::parser::{Expr, Stmt, StmtKind};

/// Known possible text values of loop variables.
type Env = HashMap<String, Vec<String>>;

pub fn check(program: &[Stmt]) -> Result<(), LangError> {
    let mut caps: Vec<(String, String)> = Vec::new();
    for stmt in program {
        if let StmtKind::Needs { action, target } = &stmt.kind {
            caps.push((action.clone(), target.clone()));
        }
    }
    let mut env = Env::new();
    check_block(program, &caps, &mut env)
}

fn check_block(
    block: &[Stmt],
    caps: &[(String, String)],
    env: &mut Env,
) -> Result<(), LangError> {
    for stmt in block {
        match &stmt.kind {
            StmtKind::Needs { .. } => {}
            StmtKind::Let(name, expr) => {
                check_expr(expr, stmt.line, caps, env)?;
                // The variable now holds something we do not track.
                env.remove(name);
            }
            StmtKind::Do(expr) | StmtKind::Verify(expr) => {
                check_expr(expr, stmt.line, caps, env)?;
            }
            StmtKind::Retry(_, body) => {
                let mut inner = env.clone();
                check_block(body, caps, &mut inner)?;
            }
            StmtKind::For(var, list, body) => {
                check_expr(list, stmt.line, caps, env)?;
                let mut inner = env.clone();
                match literal_texts(list) {
                    Some(values) => {
                        inner.insert(var.clone(), values);
                    }
                    None => {
                        inner.remove(var);
                    }
                }
                check_block(body, caps, &mut inner)?;
            }
        }
    }
    Ok(())
}

/// If the expression is a list made only of literal texts, return those texts.
fn literal_texts(expr: &Expr) -> Option<Vec<String>> {
    if let Expr::List(items) = expr {
        let mut out = Vec::new();
        for item in items {
            match item {
                Expr::Str(s) => out.push(s.clone()),
                _ => return None,
            }
        }
        Some(out)
    } else {
        None
    }
}

fn check_expr(
    expr: &Expr,
    line: usize,
    caps: &[(String, String)],
    env: &Env,
) -> Result<(), LangError> {
    match expr {
        Expr::Str(_) | Expr::Num(_) | Expr::Var(_) => Ok(()),
        Expr::List(items) => {
            for item in items {
                check_expr(item, line, caps, env)?;
            }
            Ok(())
        }
        Expr::Eq(a, b) => {
            check_expr(a, line, caps, env)?;
            check_expr(b, line, caps, env)
        }
        Expr::Call(name, args) => {
            for arg in args {
                check_expr(arg, line, caps, env)?;
            }
            if name == "read" || name == "write" || name == "fetch" {
                let values: Vec<String> = match args.first() {
                    Some(Expr::Str(p)) => vec![p.clone()],
                    Some(Expr::Var(v)) => env.get(v).cloned().unwrap_or_default(),
                    _ => Vec::new(),
                };
                for value in &values {
                    // For fetch the permission is about the domain, not the full URL.
                    let target = if name == "fetch" {
                        match net::host_of(value) {
                            Ok(host) => host,
                            Err(msg) => {
                                return Err(LangError::new(
                                    "invalid_url",
                                    Some(line),
                                    format!("fetch(\"{}\") is refused: {}", value, msg),
                                    "use a full https URL such as fetch(\"https://example.com/page\")",
                                ))
                            }
                        }
                    } else {
                        value.clone()
                    };
                    let allowed = caps.iter().any(|(a, t)| a == name && t == &target);
                    if !allowed {
                        return Err(LangError::new(
                            "capability_denied",
                            Some(line),
                            format!(
                                "{}(\"{}\") is not permitted; nothing was executed",
                                name, value
                            ),
                            format!(
                                "add this line at the top of the program: needs {}(\"{}\")",
                                name, target
                            ),
                        ));
                    }
                }
            }
            Ok(())
        }
    }
}
