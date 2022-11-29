// Tests transition between short and long encoding both ways
contract c {
    bytes data;

    fn test()  -> (u256) {
        for (uint8 i = 0; i < 33; i++) {
            data.push(bytes1(i));
        }
        for (uint8 i = 0; i < data.length; i++)
            if (data[i] != bytes1(i)) return i;
        data.pop();
        data.pop();
        for (uint8 i = 0; i < data.length; i++)
            if (data[i] != bytes1(i)) return i;
        return 0;
    }
}
// ====
// compileViaYul: also
// ----
// test() -> 0
// gas irOptimized: 157949
// gas legacy: 188576
// gas legacyOptimized: 183333