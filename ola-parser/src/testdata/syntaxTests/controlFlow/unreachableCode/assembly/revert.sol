contract C {
    fn f(uint256 y) public pure returns (uint256 x) {
        assembly {
            revert(0, 0)
            x := y
        }
    }
    fn g(uint256 y) public pure returns (uint256 x) {
        assembly {
            revert(0, 0)
        }
        x = y;
    }
}
// ----
// Warning 5740: (129-135): Unreachable code.
// Warning 5740: (274-279): Unreachable code.
