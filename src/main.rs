mod check;
mod errors;
mod interp;
mod lexer;
mod parser;

use errors::LangError;
use interp::Interp;

/// Lex, parse and run a program. Returns the interpreter (for its output and
/// audit log) together with the result.
fn run_source(src: &str) -> (Interp, Result<(), LangError>) {
    let mut interp = Interp::new();
    let result = match lexer::lex(src).and_then(parser::parse) {
        Ok(program) => match check::check(&program) {
            Ok(()) => interp.run(&program),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    };
    (interp, result)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let show_log = args.iter().any(|a| a == "--log");
    let path = match args.iter().find(|a| !a.starts_with("--")) {
        Some(p) => p.clone(),
        None => {
            eprintln!("usage: agentlang <program.agl> [--log]");
            std::process::exit(2);
        }
    };
    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cannot open '{}': {}", path, e);
            std::process::exit(2);
        }
    };

    let (interp, result) = run_source(&source);

    for line in &interp.output {
        println!("{}", line);
    }
    if show_log {
        eprintln!(
            "--- audit log (chain valid: {}) ---",
            interp::chain_is_valid(&interp.log)
        );
        for e in &interp.log {
            eprintln!(
                "#{} {} {} ok={} hash={:016x}",
                e.seq, e.action, e.target, e.ok, e.hash
            );
        }
    }
    if let Err(e) = result {
        eprintln!("{}", e.to_json());
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output_of(src: &str) -> Vec<String> {
        let (interp, result) = run_source(src);
        result.expect("program should succeed");
        interp.output
    }

    fn error_kind(src: &str) -> &'static str {
        let (_, result) = run_source(src);
        result.expect_err("program should fail").kind
    }

    #[test]
    fn prints_text() {
        assert_eq!(output_of("print(\"hello\")"), vec!["hello"]);
    }

    #[test]
    fn len_counts_characters() {
        assert_eq!(output_of("print(len(\"abc\"))"), vec!["3"]);
    }

    #[test]
    fn read_without_permission_is_denied() {
        assert_eq!(error_kind("let t = read(\"x.txt\")"), "capability_denied");
    }

    #[test]
    fn permission_is_exact() {
        let src = "needs read(\"a.txt\")\nlet t = read(\"b.txt\")";
        assert_eq!(error_kind(src), "capability_denied");
    }

    #[test]
    fn write_then_read_roundtrip() {
        let path = std::env::temp_dir().join("agentlang_test_roundtrip.txt");
        let p = path.to_string_lossy().replace('\\', "/");
        let src = format!(
            "needs write(\"{p}\")\nneeds read(\"{p}\")\nwrite(\"{p}\", \"hi\")\nprint(read(\"{p}\"))"
        );
        assert_eq!(output_of(&src), vec!["hi"]);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn needs_must_come_first() {
        let src = "print(\"x\")\nneeds read(\"a.txt\")";
        assert_eq!(error_kind(src), "needs_after_code");
    }

    #[test]
    fn retry_gives_up_after_limit() {
        let src = "needs read(\"agentlang_missing_file.txt\")\nretry 3 { let t = read(\"agentlang_missing_file.txt\") }";
        assert_eq!(error_kind(src), "retries_exhausted");
    }

    #[test]
    fn retry_does_not_repeat_denied_actions() {
        let (interp, result) = run_source("let p = \"x.txt\"\nretry 3 { let t = read(p) }");
        assert_eq!(result.expect_err("should fail").kind, "capability_denied");
        let reads = interp.log.iter().filter(|e| e.action == "read").count();
        assert_eq!(reads, 1);
    }

    #[test]
    fn retry_needs_a_bounded_number() {
        assert_eq!(error_kind("retry 99 { print(\"x\") }"), "invalid_retry");
        assert_eq!(error_kind("retry 0 { print(\"x\") }"), "invalid_retry");
    }

    #[test]
    fn verify_passes_and_fails() {
        assert_eq!(output_of("verify 1 == 1\nprint(\"ok\")"), vec!["ok"]);
        assert_eq!(error_kind("verify \"a\" == \"b\""), "verify_failed");
    }

    #[test]
    fn unknown_variable_is_reported() {
        assert_eq!(error_kind("print(nope)"), "unknown_variable");
    }

    #[test]
    fn unterminated_string_is_reported() {
        assert_eq!(error_kind("print(\"oops)"), "unterminated_string");
    }

    #[test]
    fn static_check_stops_the_program_before_anything_runs() {
        let path = std::env::temp_dir().join("agentlang_static_check.txt");
        let p = path.to_string_lossy().replace('\\', "/");
        let src = format!(
            "needs write(\"{p}\")\nprint(\"started\")\nwrite(\"{p}\", \"x\")\nlet t = read(\"nope.txt\")"
        );
        let (interp, result) = run_source(&src);
        assert_eq!(result.expect_err("should fail").kind, "capability_denied");
        assert!(interp.output.is_empty());
        assert!(interp.log.is_empty());
        assert!(!path.exists());
    }

    #[test]
    fn audit_log_detects_tampering() {
        let (interp, result) = run_source("needs read(\"a.txt\")\nverify 1 == 1");
        result.expect("program should succeed");
        assert!(interp::chain_is_valid(&interp.log));

        let mut tampered = interp.log.clone();
        tampered[0].target = "read(secret.txt)".to_string();
        assert!(!interp::chain_is_valid(&tampered));
    }
}
