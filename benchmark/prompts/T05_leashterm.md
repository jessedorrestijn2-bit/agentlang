Write a program in the language leashterm, described below, for the task below.

=== LANGUAGE DESCRIPTION ===
leashterm is a small language for AI agents. A program is a list of statements.
Whitespace and newlines do not matter. `#` starts a comment.

PERMISSIONS (required, must be at the very top, before any other statement):
    needs read("path")      # allows reading exactly this file
    needs write("path")     # allows writing exactly this file
    needs fetch("domain")   # allows https requests to exactly this domain
Every read or write must use a path that was declared with `needs`. A program
that uses anything it did not declare, or declares more than the operator allows,
is refused before it starts. Permissions cannot be declared inside blocks.

STATEMENTS:
    let name = expression          # variable
    verify a == b                  # stops the program with an error if not equal
    retry 3 { ... }                # repeats the block until it succeeds, at most N times (1 to 10)
    for item in ["x", "y"] { ... } # loops over a list written out in the program
    if a == b { ... } else { ... } # chooses a block; the else part is optional
    expression                     # for example a function call

EXPRESSIONS:
    "text"   42   name   ["a", "b"]   a == b   a != b   function(arguments)
There is no while loop and no arithmetic. Conditions must be a comparison (== or !=).

FUNCTIONS:
    print(value)          # prints one line
    len(text_or_list)     # number of characters or items
    trim(text)            # removes spaces and newlines at both ends
    concat(a, b, ...)     # joins values into one text
    read("path")          # returns the file contents as text
    write("path", text)   # writes text to the file
    fetch("https://...")  # returns the page as text

EXAMPLES:
    needs read("x/one.txt")
    needs write("x/two.txt")
    write("x/two.txt", read("x/one.txt"))

    needs read("x/one.txt")
    needs read("x/three.txt")
    for f in ["x/one.txt", "x/three.txt"] {
      print(len(read(f)))
    }

Errors are printed as JSON with an error kind, a line number, a message and a hint.

=== TASK ===
Print the number of characters in data/a.txt (the number only).

=== RULES ===
Use relative paths exactly as written above. The folders data/ and out/ already exist.
You may read only these files: data/a.txt
You may write only these files: (none)
Do not access any other file.
Output only the program code, with no explanation and no code fences.
