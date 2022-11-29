contract C {
    u256 public x;
    modifier run() {
        for (u256 i = 1; i < 10; i++) {
            if (i == 5) return;
            _;
        }
    }

    fn f() public run {
        u256 k = x;
        u256 t = k + 1;
        x = t;
    }
}

// ====
// compileViaYul: also
// compileToEwasm: also
// ----
// x() -> 0
// f() ->
// x() -> 4