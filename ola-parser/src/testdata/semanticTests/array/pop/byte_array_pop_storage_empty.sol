contract c {
    bytes data;
    fn test() public {
        data.push(0x07);
        data.push(0x05);
        data.push(0x03);
        data.pop();
        data.pop();
        data.pop();
    }
}
// ====
// compileToEwasm: also
// compileViaYul: also
// ----
// test() ->
// storageEmpty -> 1
