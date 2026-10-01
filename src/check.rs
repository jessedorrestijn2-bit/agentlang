//! Static check: runs after parsing and BEFORE anything executes.
//!
//! It collects the declared `needs` permissions and rejects the whole program if
//! a `read(...)` or `write(...)` with a literal path is not covered. Calls whose
//! path is only known at runtime (for example a variable) are still checked by
//! the interpreter while running.

use crate::errors::LangError;
use crate::parser::{Expr, Stmt, StmtKind};

pub fn check(program: &[Stmt]) -> Result<(), LangError> {
    let mut caps: Vec<(String, String)> = Vec::new();
    for stmt in program {
        if let StmtKind::Needs { action, target } = &stmt.kind {
            caps.push((action.clone(), target.clone()));
        }
    }
    check_block(program, &caps)
}

fn check_block(block: &[Stmt], caps: &[(String, String)]) -> Result<(), LangError> {
    for stmt in block {
        match &stmt.kind {
            StmtKind::Needs { .. } => {}
            StmtKind::Let(_, expr) | StmtKind::Do(expr) | StmtKind::Verify(expr) => {
                check_expr(expr, stmt.line, caps)?;
            }
            StmtKind::Retry(_, body) => check_block(body, caps)?,
        }
    }
    Ok(())
}

fn check_expr(expr: &Expr, line: usize, caps: &[(String, String)]) -> Result<(), LangError> {
    match expr {
        Expr::Str(_) | Expr::Num(_) | Expr::Var(_) => Ok(()),
        Expr::Eq(a, b) => {
            check_expr(a, line, caps)?;
            check_expr(b, line, caps)
        }
        Expr::Call(name, args) => {
            for arg in args {
                check_expr(arg, line, caps)?;
            }
            if name == "read" || name == "write" {
                if let Some(Expr::Str(path)) = args.first() {
                    let allowed = caps.iter().any(|(a, t)| a == name && t == path);
                    if !allowed {
                        return Err(LangError::new(
                            "capability_denied",
                            Some(line),
                            format!(
                                "{}(\"{}\") is not permitted; nothing was executed",
                                name, path
                            ),
                            format!(
                                "add this line at the top of the program: needs {}(\"{}\")",
                                name, path
                            ),
                        ));
                    }
                }
            }
            Ok(())
        }
    }
}
