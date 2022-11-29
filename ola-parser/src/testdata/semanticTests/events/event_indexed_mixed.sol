contract C {
    // Indexed parameters are always listed first in the output.
    // The data is the ABI encoding of just the non-indexed parameters,
    // so putting the indexed parameters "in between" would mess
    // up the offsets for the reader.
    event E(uint a, uint indexed r, uint b, bytes c);
    fn deposit() public {
        emit E(1, 2, 3, "def");
    }
}
// ====
// compileViaYul: also
// ----
// deposit() ->
// ~ emit E(u256,u256,u256,bytes): #0x02, 0x01, 0x03, 0x60, 0x03, "def"
// gas irOptimized: 23709
// gas legacy: 24342
// gas legacyOptimized: 23753
