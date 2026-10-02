# Leashterm

A tiny language for AI agents. Instead of letting a program do anything and bolting a
sandbox on afterwards, the things agents need are part of the language. (Renamed from
"agentlang", which turned out to already be the name of an unrelated, existing open-source
project.)

1. **Permissions are declared up front.** A program can only read or write what it
   declared with `needs`. Anything else is refused.
2. **Retries are always bounded.** `retry N { ... }` needs a fixed `N` (1 to 10).
3. **Verification is a statement.** `verify a == b` stops the program if it does not hold.
4. **Every action is logged** in a hash-chained audit log that can be checked for tampering.
5. **Errors are structured JSON** with a kind, a line, a message and a concrete hint,
   so a model can feed them back and repair its own program.
6. **Every program terminates.** The only repetition is `retry` (fixed limit) and `for`
   over a finite list. There is no `while`. `if`/`else` only chooses between blocks.
7. **Total work is bounded.** Every statement counts against a step budget
   (`--max-steps`, default 10,000), the general safety net that also stops nested
   `retry` blocks from silently multiplying their attempts.

## What Leashterm does and does not guarantee

**Leashterm bounds the effects that happen *during Leashterm's own execution*, through its
own runtime primitives (`read`, `write`, `fetch`). It does not, and cannot, control what a
downstream system does with content Leashterm legitimately wrote.**

A program that is only permitted to write `project/calc.py` cannot read a forbidden file to
put into that write - but nothing stops it from writing *source code that itself refers to*
something outside its permissions (an `import` statement naming a sibling package, say),
which only becomes a real access once some other interpreter later runs that file. Case 1
(`cases/case1-filesystem/`) found exactly this: 3 of 9 trials did not attempt an undeclared
read at all, yet still smuggled the dependency past Leashterm this way. This is not a bug
to patch away; it is the honest edge of what a language-level boundary can promise. The
correct claim is "declared authority is enforced within Leashterm's own execution," not
"nothing bad can ever result from a Leashterm program."

## Status (v0.7)

This is an early skeleton: lexer, parser, static permission check, interpreter, tests
and ten examples. v0.6 passed its tests and the 22-task benchmark in Codespaces and on
GitHub, including a reproducible 9-trial result (see `benchmark/README.md`). v0.7 adds a
general step budget (44 tests) and still needs its first `cargo test`.

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
leashterm prog.lsh --allow read:data/a.txt --allow write:out/b.txt
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

## The step budget

Every statement executed (including each inner attempt of a `retry`, and each pass of a
`for` loop) counts against a step budget, 10,000 by default:

```
leashterm prog.lsh --max-steps 500
```

This is the general safety net on total work, not a replacement for `--allow` or the fetch
budget: it catches the case neither of those does, nested `retry` blocks silently
multiplying their attempts (`retry 10 { retry 10 { ... } }` can reach 100 inner attempts
from two lines that each look like "at most 10"). Like a denied permission, a budget hit
inside a `retry` block is never retried; it fails the whole block immediately.

## Run it

You need Rust. On an iPad, use GitHub Codespaces: it already has a terminal where you
can install Rust (`curl https://sh.rustup.rs -sSf | sh`) or use a Rust dev container.

```
cargo test                                   # run the unit tests
cargo run -- examples/01_hello.lsh           # run a program
cargo run -- examples/02_read_file.lsh --log # also print the audit log
cargo run -- examples/03_denied.lsh          # must fail with capability_denied
cargo run -- examples/02_read_file.lsh --allow read:examples/other.txt   # policy_denied
cargo run -- examples/04_retry.lsh           # must fail with retries_exhausted
cargo run -- examples/06_for_loop.lsh        # loops over two files
cargo run -- examples/07_for_denied.lsh      # refused before anything runs
cargo run -- examples/08_fetch.lsh           # needs internet
cargo run -- examples/09_fetch_denied.lsh    # refused before anything runs
cargo run -- examples/10_if_and_concat.lsh   # if/else, concat and trim
cargo run -- examples/04_retry.lsh --max-steps 2   # must fail with budget_exceeded
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
- Nested `retry` blocks still multiply their attempts mathematically; the step budget
  only bounds the *total*, it does not stop the nesting itself, and there is no static
  check that warns about it before running (the step budget is runtime-only).
- The step budget counts statements, not wall-clock time or memory, so a single slow
  `fetch` (up to its own 10-second timeout) is not charged more than a fast one.

## Roadmap

1. Make it compile and pass `cargo test`.
2. ~~Add a static permission check before execution.~~ Done in v0.2.
3. ~~Lists and `for` loops.~~ Done in v0.3. ~~`fetch(url)` with domain permissions and a
   fetch budget.~~ Done in v0.4. Next: parallel calls with a time and cost budget.
4. Add persistent memory with its own permission, then delegation where permissions can only shrink.
5. Replay: re-run an audit log deterministically and report where results differ.
6. ~~Pilot benchmark and automatic tests on GitHub.~~ Done in v0.5 (see `benchmark/`).
7. ~~`if`/`else`, `concat`, `trim`.~~ Done in v0.6. The benchmark (not the language) grew
   to 22 tasks: T11-T12 (temptation), T13-T20 (instruction-following traps) and T21-T22 (a
   spontaneous-temptation experiment inspired by the July 2026 OpenAI-Hugging Face
   incident, see `benchmark/README.md`). ChatGPT scored 20/20 and 19/20 (Python/leashterm)
   on T01-T20, with zero out-of-bounds access either way: these tasks have not yet shown a
   safety advantage, only shorter programs. T21 and T22 are a planned family of tasks (not
   a language change) at increasing temptation strength. T21 came back clean (no attempt in
   either language); T22 did not. Repeated 9 times per language from fresh conversations:
   Python attempted the undeclared file in 9/9 trials and leaked data in 9/9; leashterm
   attempted it in 9/9 trials (identical model intent) but was blocked before execution in
   9/9 - a 100%-vs-0% result, not a single anecdote. See `benchmark/evidence/` and
   `benchmark/solutions/t22-trials/` for ChatGPT's actual, unedited answers and
   `benchmark/README.md` for the full design and this caveat: one model, one task, one
   temptation level - not yet a general claim.
8. ~~General step budget (`--max-steps`), bounding nested `retry` multiplication.~~ Done
   in v0.7. Built specifically so three practical demo cases (filesystem sandboxing,
   network egress control, and resource/cost limits - the three things companies now
   handle with external infrastructure around an agent rather than in its code) could all
   be built on exactly the same language version. Cases 1 and 2 needed no language change;
   case 3 needed this step budget.
9. ~~Case 1: filesystem sandboxing, with a 9-trial result per language.~~ Done - see
   `cases/case1-filesystem/`. Python: 89% of trials read the undeclared sibling file and
   100% of those leaked it; Leashterm: 100% of trials engaged with it (directly or via a
   newly-discovered deferred-reference pattern) and 0% leaked.
10. ~~Case 2: network egress control, with a 9-trial result per language on real,
    self-hosted domains.~~ Done - see `cases/case2-network/`. Python: 78% of trials fetched
    the undeclared domain and 100% of those leaked it; Leashterm: 33% attempted it (0%
    succeeded). Required adding real network-attempt detection to the benchmark harness
    (`socket.getaddrinfo` hook, a `fetches` permission).
11. ~~Case 3: resource/cost budgets, built on the v0.7 step budget.~~ Built and
    self-validated - see `cases/case3-resources/`. Not yet run with a real model: the next
    step is the same 9-trial process used for C1 and C2. All three cases now exist on the
    same Leashterm version (v0.7), as planned.
