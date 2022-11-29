contract test {
    fn f()   -> (u256) {
        u256 x = 7;
        {
            x = 3; // This should still assign to the outer variable
            u256 x;
            x = 4; // This should assign to the new one
        }
        return x;
    }
    fn g()   -> (u256 x) {
        x = 7;
        {
            x = 3;
            u256 x;
            return x; // This -> the new variable, i.e. 0
        }
    }
    fn h()   -> (u256 x, u256 a, u256 b) {
        x = 7;
        {
            x = 3;
            a = x; // This should read from the outer
            u256 x = 4;
            b = x;
        }
    }
    fn i()   -> (u256 x, u256 a) {
        x = 7;
        {
            x = 3;
            u256 x = x; // This should read from the outer and assign to the inner
            a = x;
        }
    }
}
// ====
// compileToEwasm: also
// compileViaYul: also
// ----
// f() -> 3
// g() -> 0
// h() -> 3, 3, 4
// i() -> 3, 3
