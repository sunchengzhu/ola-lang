pragma abicoder v1;
contract C {
    fn f(bool _b) public -> (u256) {
        if (_b) return 1;
        else return 0;
    }

    fn g(bool _in) public -> (bool _out) {
        _out = _in;
    }
}
// ====
// ABIEncoderV1Only: true
// ----
// f(bool): 0x0 -> 0x0
// f(bool): 0x1 -> 0x1
// f(bool): 0x2 -> 0x1
// f(bool): 0x3 -> 0x1
// f(bool): 0xff -> 0x1
// g(bool): 0x0 -> 0x0
// g(bool): 0x1 -> 0x1
// g(bool): 0x2 -> 0x1
// g(bool): 0x3 -> 0x1
// g(bool): 0xff -> 0x1