# fla-lab

A Rust toolchain is required to run this project. You can install one using [rustup](https://rustup.rs/).

`cargo run` to see a help message on how to run various automatas:

```
❯ cargo run

Usage: FLA <MACHINE_FILE_PATH> <COMMAND>

Commands:
  dfa   Run a DFA machine
  nfa   Run a NFA machine
  pda   Run a PDA machine
  tm    Run a Turing Machine
  help  Print this message or the help of the given subcommand(s)

Arguments:
  <MACHINE_FILE_PATH>  File path describing the DFA machine

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Assignments

1. [x] Custom file format describing automatas
2. [x] Implement the DFA runner
3. [x] Implement the two levels of the navigation game in the DFA
4. [x] Implement the NFA runner
5. [x] Implement the PDA runner
6. [x] Implement the Turing machine runner
7. [x] Vibe code various automatas (as in, the `.txt` files)

## Solutions

1.  See [`src/parser.rs`](src/parser.rs) and any of the .txt files in the root directory for examples
2.  See [`src/machine/dfa.rs`](src/machine/dfa.rs)
3.  See [`dfa_level1_escape.txt`](./dfa_level1_escape.txt) and [`dfa_level2_escape_with_key.txt`](./dfa_level2_escape_with_key.txt). Refer to [#DFA Tests](#dfa-test) for winning/losing examples.
4.  See [`src/machine/nfa.rs`](src/machine/nfa.rs)
5.  See [`src/machine/pda.rs`](src/machine/pda.rs)
6.  See [`src/machine/tm.rs`](src/machine/tm.rs)
7.  See the following:

    a. See [`tm_palindrome.txt`](./tm_palindrome.txt):

    ```
           cargo run -- tm_palindrome.txt tm "abba"
           # Expected: ACCEPTED (palindrome)

           cargo run -- tm_palindrome.txt tm "aba"
           # Expected: ACCEPTED (palindrome)

           cargo run -- tm_palindrome.txt tm ""
           # Expected: ACCEPTED (empty string is a palindrome)

           cargo run -- tm_palindrome.txt tm "abab"
           # Expected: REJECTED (not a palindrome)

           cargo run -- tm_palindrome.txt tm "aabb"
           # Expected: REJECTED (not a palindrome)
    ```

    b. See [`tm_video_memory.txt`](./tm_video_memory.txt):

    ```
           cargo run -- tm_video_memory.txt tm "01_v**_v"
           # Expected: ACCEPTED (Tape: _**v01_v)

           cargo run -- tm_video_memory.txt tm "101_v_____v"
           # Expected: ACCEPTED (Tape: ____v101__v)

           cargo run -- tm_video_memory.txt tm "_v___v"
           # Expected: ACCEPTED (Tape: _v___v - nothing to move)

           cargo run -- tm_video_memory.txt tm "0v___v"
           # Expected: ACCEPTED (Tape: _v0__v)
    ```

    c. See [`tm_unary_addition.txt`](./tm_unary_addition.txt):

    ```
    cargo run -- tm_unary_addition.txt tm "11+111"
    # Expected: ACCEPTED (Tape: 11111_)

    cargo run -- tm_unary_addition.txt tm "1+1"
    # Expected: ACCEPTED (Tape: 11_)

    cargo run -- tm_unary_addition.txt tm "+11"
    # Expected: ACCEPTED (Tape: 11_)

    cargo run -- tm_unary_addition.txt tm "11+"
    # Expected: ACCEPTED (Tape: 11_)

    cargo run -- tm_unary_addition.txt tm "111"
    # Expected: ACCEPTED (Tape: 111_)

    cargo run -- tm_unary_addition.txt tm "+"
    # Expected: ACCEPTED (Tape: _)

    cargo run -- tm_unary_addition.txt tm ""
    # Expected: ACCEPTED (Tape: _)

    cargo run -- tm_unary_addition.txt tm "1+1+1"
    # Expected: REJECTED
    ```

## Misc tests for all the `.txt` files present

### DFA test

```
cargo run -- dfa.txt dfa 1

# Expected: ACCEPTED
```

#### DFA Game

```
cargo run -- dfa_level1_escape.txt dfa "ULU"
# Expected: ACCEPTED (escaped: Entrance -> Hallway -> Library -> Exit)

cargo run -- dfa_level1_escape.txt dfa "UUL"
# Expected: REJECTED (stuck: Entrance -> Hallway -> Kitchen -> InvalidState trying to go L)

cargo run -- dfa_level1_escape.txt dfa "URU"
# Expected: REJECTED (stuck: Entrance -> Hallway -> SecretRoom -> InvalidState trying to go U)
```

```
cargo run -- dfa_level2_escape_with_key.txt dfa "UUPDLU"
# Expected: ACCEPTED (Entrance -> Hallway_NoKey -> Kitchen_NoKey -> P -> Kitchen_HasKey -> Hallway_HasKey -> Library_HasKey -> Exit_HasKey)

cargo run -- dfa_level2_escape_with_key.txt dfa "ULU"
# Expected: REJECTED (reached exit without key: Entrance -> Hallway_NoKey -> Library_NoKey -> tries U to Exit -> InvalidState)

cargo run -- dfa_level2_escape_with_key.txt dfa "UUDLU"
# Expected: REJECTED (reached exit without key, same as above but didn't try PU: Entrance -> Hallway_NoKey -> Kitchen_NoKey -> Hallway_NoKey -> Library_NoKey -> tries U to Exit -> InvalidState)

cargo run -- dfa_level2_escape_with_key.txt dfa "UUPUDRL"
# Expected: REJECTED (stuck in SecretRoom with key: Entrance -> Hallway_NoKey -> Kitchen_NoKey -> PU -> Kitchen_HasKey -> Hallway_HasKey -> SecretRoom_HasKey -> tries L to Hallway -> Hallway_HasKey, but this is not an exit path)

cargo run -- dfa_level2_escape_with_key.txt dfa "L"
# Expected: REJECTED (invalid move from Entrance: Entrance -> InvalidState)
```

### NFA tests

```
cargo run -- nfa_accept.txt nfa "ababab"

# Expected: ACCEPTED

cargo run -- nfa_accept.txt nfa "aba"

# Expected: REJECTED

cargo run -- nfa_complex.txt nfa "aabaa"

# Expected: ACCEPTED (contains "ab")
```

### PDA tests

```
cargo run -- pda_anbn.txt pda "aabb"

# Expected: ACCEPTED

cargo run -- pda_palindrome.txt pda "abcba"

# Expected: ACCEPTED
```

### TM tests

```
cargo run -- tm_palindrome.txt tm "abba"

# Expected: ACCEPTED (palindrome)

cargo run -- tm_palindrome.txt tm "aba"

# Expected: ACCEPTED (palindrome)

cargo run -- tm_palindrome.txt tm ""

# Expected: ACCEPTED (empty string is a palindrome)

cargo run -- tm_palindrome.txt tm "abab"

# Expected: REJECTED (not a palindrome)

cargo run -- tm_palindrome.txt tm "aabb"

# Expected: REJECTED (not a palindrome)
```

```
cargo run -- tm_video_memory.txt tm "01_v\_\_\_v"

# Expected: ACCEPTED (Tape: \_\_\_v01_v)

cargo run -- tm_video_memory.txt tm "101_v**\_**v"

# Expected: ACCEPTED (Tape: \_**\_v101**v)

cargo run -- tm_video_memory.txt tm "\_v\_\_\_v"

# Expected: ACCEPTED (Tape: \_v\_\_\_v - nothing to move)

cargo run -- tm_video_memory.txt tm "0v\_\_\_v"

# Expected: ACCEPTED (Tape: \_v0\_\_v)
```

```
cargo run -- tm_unary_addition.txt tm "11+111"
# Expected: ACCEPTED (Tape: 11111_)

cargo run -- tm_unary_addition.txt tm "1+1"
# Expected: ACCEPTED (Tape: 11_)

cargo run -- tm_unary_addition.txt tm "+11"
# Expected: ACCEPTED (Tape: 11_)

cargo run -- tm_unary_addition.txt tm "11+"
# Expected: ACCEPTED (Tape: 11_)

cargo run -- tm_unary_addition.txt tm "111"
# Expected: ACCEPTED (Tape: 111_)

cargo run -- tm_unary_addition.txt tm "+"
# Expected: ACCEPTED (Tape: _)

cargo run -- tm_unary_addition.txt tm ""
# Expected: ACCEPTED (Tape: _)

cargo run -- tm_unary_addition.txt tm "1+1+1"
# Expected: REJECTED
```
