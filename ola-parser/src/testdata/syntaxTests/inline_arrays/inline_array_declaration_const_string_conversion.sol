contract C {
    fn f() public returns (string memory) {
        string[2] memory z = ["Hello", "World"];
        return (z[0]);
    }
}
// ----
// Warning 2018: (17-140): fn state mutability can be restricted to pure