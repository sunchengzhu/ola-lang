contract receiver {
    uint public received;
    fn recv(uint x) public { received += x + 1; }
    fallback() external { received = 0x80; }
}
contract sender {
    constructor() { rec = new receiver(); }
    fallback() external { savedData1 = savedData2 = msg.data; }
    fn forward(bool selector) public returns (bool) {
        if (selector) { address(rec).call(savedData1); delete savedData1; }
        else { address(rec).call(savedData2); delete savedData2; }
        return true;
    }
    fn val() public returns (uint) { return rec.received(); }
    receiver rec;
    bytes savedData1;
    bytes savedData2;
}
// ====
// compileToEwasm: false
// compileViaYul: also
// ----
// (): 7 ->
// gas irOptimized: 110954
// gas legacy: 111071
// gas legacyOptimized: 111016
// val() -> 0
// forward(bool): true -> true
// val() -> 0x80
// forward(bool): false -> true
// val() -> 0x80
// forward(bool): true -> true
// val() -> 0x80
