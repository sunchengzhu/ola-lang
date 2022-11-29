contract C {
    uint8[] tester;

    fn f()  -> (uint8[5] memory) {
        return ([1, 2, 3, 4, 5]);
    }

    fn test()  -> (uint8, uint8, uint8, uint8, uint8) {
        tester = f();
        return (tester[0], tester[1], tester[2], tester[3], tester[4]);
    }
}

// ====
// compileToEwasm: also
// compileViaYul: also
// ----
// f() -> 1, 2, 3, 4, 5