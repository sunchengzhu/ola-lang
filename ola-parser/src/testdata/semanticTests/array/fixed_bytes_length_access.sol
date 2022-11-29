contract C {
    bytes1 a;

    fn f(bytes32 x)  -> (u256, u256, u256) {
        return (x.length, bytes16(uint128(2)).length, a.length + 7);
    }
}
// ====
// compileToEwasm: also
// compileViaYul: also
// ----
// f(bytes32): "789" -> 32, 16, 8
