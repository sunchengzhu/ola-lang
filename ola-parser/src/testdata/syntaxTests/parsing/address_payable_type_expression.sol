contract C {
    fn f() public pure {
        address payable;
    }
}
// ----
// ParserError 2314: (67-68): Expected identifier but got ';'
