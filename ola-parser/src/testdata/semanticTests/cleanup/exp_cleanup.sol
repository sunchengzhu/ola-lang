contract C {
    fn f()  -> (u256 x) {
        unchecked {
            uint8 y = uint8(2)**uint8(8);
            return 0**y;
        }
    }
}

// ====
// compileViaYul: also
// compileToEwasm: also
// ----
// f() -> 0x1