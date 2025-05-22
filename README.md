# lfa-lab

`cargo run` to see a help message on how to run various automatas.

## Assignments

1. [x] Custom file format describing automatas: see any of the `*.txt` files in the root (and it's corresponding parser!)
       see [`src/automata.rs`](src/parser.rs) for the parser
2. [x] Implement the DFA runner; see [`src/machine/dfa.rs`](src/machine/dfa.rs) for the DFA struct and its methods
3. [] Implement the two levels of the navigation game in the DFA (TODO)
4. [x] Implement the NFA runner; see [`src/machine/nfa.rs`](src/machine/nfa.rs) for the NFA struct and its methods
5. [x] Implement the PDA runner; see [`src/machine/pda.rs`](src/machine/pda.rs) for the PDA struct and its methods
6. [] Implement the Turing machine runner (TODO)
7. [] Implement the video space simulation Turing Machine (TODO)
8. [] Vibe code various automatas (as in, the `.txt` files) (TODO)

## DFA test

```
cargo run -- dfa.txt dfa 1
# Expected: ACCEPTED
```

## NFA tests

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
