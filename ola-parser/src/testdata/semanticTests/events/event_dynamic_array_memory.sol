contract C {
    event E(uint[]);
    fn createEvent(uint x) public {
        uint[] memory arr = new uint[](3);
        arr[0] = x;
        arr[1] = x + 1;
        arr[2] = x + 2;
        emit E(arr);
    }
}
// ====
// compileViaYul: also
// ----
// createEvent(u256): 42 ->
// ~ emit E(u256[]): 0x20, 0x03, 0x2a, 0x2b, 0x2c
