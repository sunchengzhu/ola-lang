contract C {
    struct S { uint x; }
    fn f() public pure {
        S[] memory s;
        abi.encodePacked(s);
    }
}
// ----
// TypeError 9578: (116-117): Type not supported in packed mode.
