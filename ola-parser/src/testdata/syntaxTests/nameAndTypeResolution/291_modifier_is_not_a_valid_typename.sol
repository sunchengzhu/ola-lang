contract test {
    modifier mod() { _; }

    fn f() public {
        mod g;
    }
}
// ----
// TypeError 5172: (77-80): Name has to refer to a struct, enum or contract.