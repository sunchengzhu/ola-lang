contract C {
    fn g()  -> (u256 a, u256 b, u256 c) {
        a = 1;
        b = 2;
        c = 3;
    }

    fn h()  -> (u256 a, u256 b, u256 c, u256 d) {
        a = 1;
        b = 2;
        c = 3;
        d = 4;
    }

    fn f1()  -> (bool) {
        (u256 x, u256 y, u256 z) = g();
        if (x != 1 || y != 2 || z != 3) return false;
        (, u256 a, ) = g();
        if (a != 2) return false;
        (u256 b, , ) = g();
        if (b != 1) return false;
        (, , u256 c) = g();
        if (c != 3) return false;
        return true;
    }

    fn f2()  -> (bool) {
        (u256 a1, , u256 a3, ) = h();
        if (a1 != 1 || a3 != 3) return false;
        (u256 b1, u256 b2, , ) = h();
        if (b1 != 1 || b2 != 2) return false;
        (, u256 c2, u256 c3, ) = h();
        if (c2 != 2 || c3 != 3) return false;
        (, , u256 d3, u256 d4) = h();
        if (d3 != 3 || d4 != 4) return false;
        (u256 e1, , u256 e3, u256 e4) = h();
        if (e1 != 1 || e3 != 3 || e4 != 4) return false;
        return true;
    }

    fn f()  -> (bool) {
        return f1() && f2();
    }
}
// ====
// compileToEwasm: also
// compileViaYul: also
// ----
// f() -> true
