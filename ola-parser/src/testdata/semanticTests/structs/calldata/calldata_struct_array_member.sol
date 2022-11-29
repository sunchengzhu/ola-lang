pragma abicoder v2;


contract C {
    struct S {
        u256 a;
        u256[2] b;
        u256 c;
    }

    fn f(S calldata s)
        external
        pure
        -> (u256 a, u256 b0, u256 b1, u256 c)
    {
        a = s.a;
        b0 = s.b[0];
        b1 = s.b[1];
        c = s.c;
    }
}
// ====
// compileViaYul: also
// ----
// f((u256,u256[2],u256)): 42, 1, 2, 23 -> 42, 1, 2, 23
