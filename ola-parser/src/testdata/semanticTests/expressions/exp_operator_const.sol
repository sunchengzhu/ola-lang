contract test {
    fn f()  ->(u256 d) { return 2 ** 3; }
}
// ====
// compileToEwasm: also
// compileViaYul: also
// ----
// f() -> 8
