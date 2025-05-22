# lfa-lab

`cargo run` to see a help message on how to run various automatas.

## Assignments

1. [x] Custom file format describing automatas: see any of the `*.txt` files in the root (and it's corresponding parser!)
2. [x] Implement the DFA runner
3. [] Implement the two levels of the navigation game in the DFA
4. [x] Implement the NFA runner
5. [x] Implement the PDA runner
6. [x] Implement the Turing machine runner
7. [] Implement the video space simulation Turing Machine (TODO)
8. [x] Vibe code various automatas (as in, the `.txt` files)

## Solutions

1. See [`src/automata.rs`](src/parser.rs)
2. See [`src/machine/dfa.rs`](src/machine/dfa.rs)
3. TODO
4. See [`src/machine/nfa.rs`](src/machine/nfa.rs)
5. See [`src/machine/pda.rs`](src/machine/pda.rs)
6. See [`src/machine/tm.rs`](src/machine/tm.rs)
7. TODO
8. See [`tm_palindrome.txt`](./tm_palindrome.txt):

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

## Misc tests for all the `.txt` files present

### DFA test

```
cargo run -- dfa.txt dfa 1
# Expected: ACCEPTED
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
