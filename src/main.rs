mod check;
mod errors;
mod interp;
mod lexer;
mod net;
mod parser;

use errors::LangError;
use interp::Interp;

/// Lex, parse and run a program (no operator policy).
#[cfg(test)]
fn run_source(src: &str) -> (Interp, Result<(), LangError>) {
    run_program(src, net::real_fetch, None)
}

/// Same as `run_source`, but with a custom fetch function (used by tests).
#[cfg(test)]
fn run_source_with(src: &str, fetch: net::FetchFn) -> (Interp, Result<(), LangError>) {
    run_program(src, fetch, None)
}

/// The full pipeline: lex, parse, operator policy, static check, run.
/// Returns the interpreter (for its output and audit log) with the result.
fn run_program(
    src: &str,
    fetch: net::FetchFn,
    policy: Option<Vec<(String, String)>>,
) -> (Interp, Result<(), LangError>) {
    let mut interp = Interp::new();
    interp.fetch_impl = fetch;
    let result = match lexer::lex(src).and_then(parser::parse) {
        Ok(program) => {
            let policy_result = match &policy {
                Some(p) => check::check_policy(&program, p),
                None => Ok(()),
            };
            match policy_result.and_then(|_| check::check(&program)) {
                Ok(()) => interp.run(&program),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    };
    (interp, result)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut path: Option<String> = None;
    let mut show_log = false;
    let mut allowed: Vec<(String, String)> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--log" => show_log = true,
            "--allow" => {
                i += 1;
                let value = args.get(i).cloned().unwrap_or_default();
                match value.split_once(':') {
                    Some((a, t)) if !a.is_empty() && !t.is_empty() => {
                        allowed.push((a.to_string(), t.to_string()));
                    }
                    _ => {
                        eprintln!("--allow needs a value like read:data/a.txt");
                        std::process::exit(2);
                    }
                }
            }
            flag if flag.starts_with("--") => {
                eprintln!("unknown option '{}'", flag);
                std::process::exit(2);
            }
            other => path = Some(other.to_string()),
        }
        i += 1;
    }

    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("usage: agentlang <program.agl> [--log] [--allow action:target]...");
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

    // Without --allow flags the program's own `needs` lines are the only limit.
    // With them, the operator's list is the hard limit.
    let policy = if allowed.is_empty() { None } else { Some(allowed) };
    let (interp, result) = run_program(&source, net::real_fetch, policy);

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
    fn for_loop_visits_every_item() {
        assert_eq!(
            output_of("for x in [\"a\", \"b\", \"c\"] { print(x) }"),
            vec!["a", "b", "c"]
        );
    }

    #[test]
    fn len_counts_list_items() {
        assert_eq!(output_of("print(len([\"a\", \"b\"]))"), vec!["2"]);
    }

    #[test]
    fn for_loop_can_write_several_files() {
        let a = std::env::temp_dir().join("agentlang_loop_a.txt");
        let b = std::env::temp_dir().join("agentlang_loop_b.txt");
        let pa = a.to_string_lossy().replace('\\', "/");
        let pb = b.to_string_lossy().replace('\\', "/");
        let src = format!(
            "needs write(\"{pa}\")\nneeds write(\"{pb}\")\nfor p in [\"{pa}\", \"{pb}\"] {{ write(p, \"x\") }}\nprint(\"done\")"
        );
        assert_eq!(output_of(&src), vec!["done"]);
        assert!(a.exists());
        assert!(b.exists());
        let _ = std::fs::remove_file(a);
        let _ = std::fs::remove_file(b);
    }

    #[test]
    fn static_check_looks_through_literal_lists() {
        let src = "needs read(\"a.txt\")\nprint(\"started\")\nfor p in [\"a.txt\", \"b.txt\"] { let t = read(p) }";
        let (interp, result) = run_source(src);
        assert_eq!(result.expect_err("should fail").kind, "capability_denied");
        assert!(interp.output.is_empty());
        assert!(interp.log.is_empty());
    }

    #[test]
    fn for_needs_a_list() {
        assert_eq!(error_kind("for x in \"abc\" { print(x) }"), "not_a_list");
    }

    #[test]
    fn needs_inside_for_is_rejected() {
        assert_eq!(
            error_kind("for x in [\"a\"] { needs read(\"a\") }"),
            "needs_in_block"
        );
    }

    fn fake_fetch(url: &str) -> Result<String, String> {
        Ok(format!("page from {}", url))
    }

    fn failing_fetch(_url: &str) -> Result<String, String> {
        Err("boom".to_string())
    }

    #[test]
    fn host_of_accepts_normal_urls() {
        assert_eq!(net::host_of("https://example.com").unwrap(), "example.com");
        assert_eq!(net::host_of("https://Example.COM/a?b=c#d").unwrap(), "example.com");
        assert_eq!(net::host_of("https://example.com:8443/x").unwrap(), "example.com");
    }

    #[test]
    fn host_of_refuses_tricks() {
        assert!(net::host_of("http://example.com").is_err());
        assert!(net::host_of("https://example.com@evil.com/").is_err());
        assert!(net::host_of("https://example.com\\@evil.com/").is_err());
        assert!(net::host_of("https://evil.com\\.example.com/").is_err());
        assert!(net::host_of("https://").is_err());
        assert!(net::host_of("https://exa mple.com").is_err());
    }

    #[test]
    fn fetch_with_permission_returns_the_page() {
        let src = "needs fetch(\"example.com\")\nprint(fetch(\"https://example.com/a\"))";
        let (interp, result) = run_source_with(src, fake_fetch);
        result.expect("program should succeed");
        assert_eq!(interp.output, vec!["page from https://example.com/a"]);
        let last = interp.log.last().unwrap();
        assert_eq!(last.action, "fetch");
        assert_eq!(last.target, "example.com");
    }

    #[test]
    fn fetch_of_another_domain_is_refused_before_running() {
        let src = "needs fetch(\"example.com\")\nprint(\"started\")\nlet p = fetch(\"https://evil.com/x\")";
        let (interp, result) = run_source_with(src, fake_fetch);
        assert_eq!(result.expect_err("should fail").kind, "capability_denied");
        assert!(interp.output.is_empty());
        assert!(interp.log.is_empty());
    }

    #[test]
    fn a_subdomain_needs_its_own_permission() {
        let src = "needs fetch(\"example.com\")\nlet p = fetch(\"https://api.example.com\")";
        let (_, result) = run_source_with(src, fake_fetch);
        assert_eq!(result.expect_err("should fail").kind, "capability_denied");
    }

    #[test]
    fn user_info_trick_and_plain_http_are_invalid_urls() {
        let a = "needs fetch(\"example.com\")\nlet p = fetch(\"https://example.com@evil.com/\")";
        let b = "needs fetch(\"example.com\")\nlet p = fetch(\"http://example.com\")";
        assert_eq!(run_source_with(a, fake_fetch).1.expect_err("fail").kind, "invalid_url");
        assert_eq!(run_source_with(b, fake_fetch).1.expect_err("fail").kind, "invalid_url");
    }

    #[test]
    fn static_check_covers_fetch_in_a_loop() {
        let src = "needs fetch(\"example.com\")\nprint(\"started\")\nfor u in [\"https://example.com/1\", \"https://evil.com/2\"] { let p = fetch(u) }";
        let (interp, result) = run_source_with(src, fake_fetch);
        assert_eq!(result.expect_err("should fail").kind, "capability_denied");
        assert!(interp.output.is_empty());
        assert!(interp.log.is_empty());
    }

    #[test]
    fn failed_fetches_are_retried_and_logged() {
        let src = "needs fetch(\"example.com\")\nretry 3 { let p = fetch(\"https://example.com\") }";
        let (interp, result) = run_source_with(src, failing_fetch);
        assert_eq!(result.expect_err("should fail").kind, "retries_exhausted");
        let failed = interp
            .log
            .iter()
            .filter(|e| e.action == "fetch" && !e.ok)
            .count();
        assert_eq!(failed, 3);
    }

    #[test]
    fn fetch_budget_stops_the_program() {
        let src = "needs fetch(\"example.com\")\nfor u in [\"https://example.com/1\", \"https://example.com/2\", \"https://example.com/3\"] { let p = fetch(u) }";
        let program = parser::parse(lexer::lex(src).unwrap()).unwrap();
        check::check(&program).unwrap();
        let mut interp = Interp::new();
        interp.fetch_impl = fake_fetch;
        interp.max_fetches = 2;
        let err = interp.run(&program).expect_err("should hit the budget");
        assert_eq!(err.kind, "budget_exceeded");
    }

    fn policy(items: &[(&str, &str)]) -> Option<Vec<(String, String)>> {
        Some(items.iter().map(|(a, t)| (a.to_string(), t.to_string())).collect())
    }

    #[test]
    fn operator_policy_blocks_self_granted_permissions() {
        let src = "needs read(\"data/secret.txt\")\nprint(\"started\")\nprint(read(\"data/secret.txt\"))";
        let (interp, result) = run_program(src, fake_fetch, policy(&[("read", "data/public.txt")]));
        assert_eq!(result.expect_err("should fail").kind, "policy_denied");
        assert!(interp.output.is_empty());
        assert!(interp.log.is_empty());
    }

    #[test]
    fn operator_policy_allows_what_it_lists() {
        let src = "needs read(\"data/missing_for_test.txt\")\nretry 1 { let t = read(\"data/missing_for_test.txt\") }";
        let (_, result) = run_program(src, fake_fetch, policy(&[("read", "data/missing_for_test.txt")]));
        // The policy lets the program start; it then fails only because the file does not exist.
        assert_eq!(result.expect_err("should fail").kind, "retries_exhausted");
    }

    #[test]
    fn operator_policy_hint_lists_what_is_allowed() {
        let src = "needs write(\"out/x.txt\")";
        let (_, result) = run_program(src, fake_fetch, policy(&[("read", "data/a.txt")]));
        let err = result.expect_err("should fail");
        assert_eq!(err.kind, "policy_denied");
        assert!(err.hint.contains("read(\"data/a.txt\")"));
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
