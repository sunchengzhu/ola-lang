contract C {
    fn f(bytes20 x) public pure returns (address payable) {
        return payable(address(x));
    }
}
// ----
