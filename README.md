# lfa-lab

`cargo run` to see a help message.

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
