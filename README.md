# agentlang (working name)

A tiny language for AI agents. Instead of letting a program do anything and bolting a
sandbox on afterwards, the things agents need are part of the language:

1. **Permissions are declared up front.** A program can only read or write what it
   declared with `needs`. Anything else is refused.
2. **Retries are always bounded.** `retry N { ... }` needs a fixed `N` (1 to 10).
3. **Verification is a statement.** `verify a == b` stops the program if it does not hold.
4. **Every action is logged** in a hash-chained audit log that can be checked for tampering.
5. **Errors are structured JSON** with a kind, a line, a message and a concrete hint,
   so a model can feed them back and repair its own program.
6. **There are no loops yet**, so every program terminates.

## Status (v0.2)

This is an early skeleton: lexer, parser, static permission check, interpreter, tests
and five examples. v0.1 compiled and passed its 13 tests in Codespaces; the static
check added in v0.2 still needs its first `cargo test`.

## Syntax

```
# comment
needs read("notes.txt")        # permission (only allowed at the top)
needs write("out.txt")

let text = read("notes.txt")   # variables
print(text)                    # builtins: print, len, read, write
verify len(text) == 10         # stop the program if false

retry 3 {                      # bounded retry, never repeats a missing permission
  let t = read("maybe.txt")
}
```

Values are text, numbers and booleans. `==` compares two values.

## Run it

You need Rust. On an iPad, use GitHub Codespaces: it already has a terminal where you
can install Rust (`curl https://sh.rustup.rs -sSf | sh`) or use a Rust dev container.

```
cargo test                                   # run the unit tests
cargo run -- examples/01_hello.agl           # run a program
cargo run -- examples/02_read_file.agl --log # also print the audit log
cargo run -- examples/03_denied.agl          # must fail with capability_denied
cargo run -- examples/04_retry.agl           # must fail with retries_exhausted
```

Example of a refused program (stderr):

```
{"error":"capability_denied","line":4,"message":"read(\"examples/secret.txt\") is not permitted","hint":"add this line at the top of the program: needs read(\"examples/secret.txt\")"}
```

## Known limitations

- The audit log uses Rust's `DefaultHasher`. That is a placeholder, not secure. Use SHA-256.
- Permissions are checked before running for literal paths (`src/check.rs`). Paths that
  are only known at runtime, such as a variable, are still checked while running.
- No network access, no parallel calls, no memory, no sub-agents yet.
- Nested `retry` blocks multiply their attempts.

## Roadmap

1. Make it compile and pass `cargo test`.
2. ~~Add a static permission check before execution.~~ Done in v0.2.
3. Add `fetch(url)` with domain permissions, then parallel calls with a time and cost budget.
4. Add persistent memory with its own permission, then delegation where permissions can only shrink.
5. Replay: re-run an audit log deterministically and report where results differ.
6. Benchmark: the same agent tasks in this language and in Python, measuring errors and tokens.
