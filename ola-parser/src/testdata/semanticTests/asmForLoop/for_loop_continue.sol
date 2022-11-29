contract C {
    fn f()  -> (u256 k) {
        assembly {
            for {let i := 0} lt(i, 10) { i := add(i, 1) }
            {
                if eq(mod(i, 2), 0) { continue }
                k := add(k, 1)
            }
        }
    }
}
// ====
// compileToEwasm: also
// compileViaYul: also
// ----
// f() -> 5