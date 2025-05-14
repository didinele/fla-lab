# lfa-lab

`cargo run` to see a help message.

## DFA test

```
❯ cargo run -- dfa.txt dfa 1
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/LFA dfa.txt dfa 1`
Input is ACCEPTED
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
