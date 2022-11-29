pragma abicoder v2;
contract C {
    fn f(uint16 a, uint16 b)  -> (uint16) {
        return a + b;
    }
}
// ====
// compileViaYul: also
// ----
// f(uint16,uint16): 65534, 0 -> 0xfffe
// f(uint16,uint16): 65536, 0 -> FAILURE
// f(uint16,uint16): 65535, 0 -> 0xffff
// f(uint16,uint16): 65535, 1 -> FAILURE, hex"4e487b71", 0x11