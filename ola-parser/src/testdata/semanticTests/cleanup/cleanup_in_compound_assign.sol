contract C {
    fn test() public -> (u256, u256) {
        uint32 a = 0xffffffff;
        uint16 x = uint16(a);
        uint16 y = x;
        x /= 0x100;
        y = y / 0x100;
        return (x, y);
    }
}
// ====
// compileToEwasm: also
// compileViaYul: also
// ----
// test() -> 0xff, 0xff
