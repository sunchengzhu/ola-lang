contract Test {
    fn set(bytes memory _data, uint256 i)
        public
        returns (uint256 l, bytes1 c)
    {
        l = _data.length;
        c = _data[i];
    }
}
// ====
// compileViaYul: also
// ----
// set(bytes,uint256): 0x40, 0x03, 0x08, "abcdefgh" -> 0x08, "d"