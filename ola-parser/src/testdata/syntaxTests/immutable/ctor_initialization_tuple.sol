contract C {
    uint immutable x;
    uint immutable y;
    constructor() {
        (x, y) = f();
    }

    fn f() internal pure returns(uint _x, uint _y) {
        _x = 3;
        _y = 4;
    }
}
// ----
