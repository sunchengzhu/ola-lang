contract A {
    constructor(mapping(u256 => u256)[] storage) {}
}
// ----
// TypeError 3644: (30-63): This parameter has a type that can only be used internally. You can make the contract abstract to avoid this problem.
