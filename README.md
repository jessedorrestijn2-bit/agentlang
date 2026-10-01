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
6. **Every program terminates.** The only repetition is `retry` (fixed limit) and `for`
   over a finite list. There is no `while`. `if`/`else` only chooses between blocks.

## Status (v0.6)

This is an early skeleton: lexer, parser, static permission check, interpreter, tests
and ten examples. v0.5 passed its tests and the benchmark in Codespaces and on GitHub.
v0.6 adds `if`/`else`, `!=`, `concat` and `trim` (40 tests) and still needs its first
`cargo test`. A first pilot run (one model, one try) scored Python 10/10 and agentlang
8/10; the two misses were exactly the features v0.6 now adds.

## Syntax

```
# comment
needs read("notes.txt")        # permission (only allowed at the top)
needs write("out.txt")
needs fetch("example.com")     # network permission is per domain

let text = read("notes.txt")   # variables
print(text)                    # builtins: print, len, trim, concat, read, write, fetch
verify len(text) == 10         # stop the program if false

retry 3 {                      # bounded retry, never repeats a missing permission
  let t = read("maybe.txt")
}

let page = fetch("https://example.com/page")   # https only, domain must be declared

if trim(text) == "yes" {       # chooses a block; else is optional; conditions are == or !=
  print(concat("got: ", text))
} else {
  print("no")
}

for f in ["a.txt", "b.txt"] {  # loops over a finite list, always stops
  print(read(f))
}
```

Values are text, numbers, booleans and lists. `==` compares two values.

## Operator policy: who grants the permissions

A program's `needs` lines are requests. Without more, a program could simply grant itself
anything. So the person or system that runs it can set a hard limit:

```
agentlang prog.agl --allow read:data/a.txt --allow write:out/b.txt
```

If any `--allow` is given, a program that asks (with `needs`) for something not on that
list is refused before it starts, with `policy_denied` and a hint that lists what is
allowed. Without `--allow`, the program's own `needs` lines are the only limit.

## How `fetch` stays safe

- Only `https://` URLs. The permission names a domain: `needs fetch("example.com")`.
- A subdomain such as `api.example.com` needs its own permission.
- Tricks like `https://example.com@evil.com/` are refused as invalid URLs.
- Redirects are blocked, because they could leave the permitted domain.
- 10 second timeout and at most 50 fetches per run (a simple cost budget).
- The audit log records only the domain, not the full URL (which may contain secrets).
- The tests use a fake fetch function, so they never need the internet.

## Run it

You need Rust. On an iPad, use GitHub Codespaces: it already has a terminal where you
can install Rust (`curl https://sh.rustup.rs -sSf | sh`) or use a Rust dev container.

```
cargo test                                   # run the unit tests
cargo run -- examples/01_hello.agl           # run a program
cargo run -- examples/02_read_file.agl --log # also print the audit log
cargo run -- examples/03_denied.agl          # must fail with capability_denied
cargo run -- examples/02_read_file.agl --allow read:examples/other.txt   # policy_denied
cargo run -- examples/04_retry.agl           # must fail with retries_exhausted
cargo run -- examples/06_for_loop.agl        # loops over two files
cargo run -- examples/07_for_denied.agl      # refused before anything runs
cargo run -- examples/08_fetch.agl           # needs internet
cargo run -- examples/09_fetch_denied.agl    # refused before anything runs
cargo run -- examples/10_if_and_concat.agl   # if/else, concat and trim
```

Example of a refused program (stderr):

```
{"error":"capability_denied","line":4,"message":"read(\"examples/secret.txt\") is not permitted","hint":"add this line at the top of the program: needs read(\"examples/secret.txt\")"}
```

## Known limitations

- The audit log uses Rust's `DefaultHasher`. That is a placeholder, not secure. Use SHA-256.
- Permissions are checked before running for literal paths and for loop variables over a
  literal list (`src/check.rs`). Other paths, such as a variable that holds a result, are
  still only checked while running.
- No parallel calls, no memory, no sub-agents yet. `fetch` only does GET and has no
  wildcard domains. Redirects are blocked rather than followed.
- Nested `retry` blocks multiply their attempts.

## Roadmap

1. Make it compile and pass `cargo test`.
2. ~~Add a static permission check before execution.~~ Done in v0.2.
3. ~~Lists and `for` loops.~~ Done in v0.3. ~~`fetch(url)` with domain permissions and a
   fetch budget.~~ Done in v0.4. Next: parallel calls with a time and cost budget.
4. Add persistent memory with its own permission, then delegation where permissions can only shrink.
5. Replay: re-run an audit log deterministically and report where results differ.
6. ~~Pilot benchmark and automatic tests on GitHub.~~ Done in v0.5 (see `benchmark/`).
7. ~~`if`/`else`, `concat`, `trim`.~~ Done in v0.6, after the first pilot run showed agents
   missed them. Next: rerun the benchmark with the new tasks T11 and T12, with more models
   and several tries per task.
