// This used to work in pre-0.6.0.
library Lib {
    fn min(uint, uint) public returns (uint);
}
contract Test {
    fn f() public {
        uint t = Lib.min(12, 7);
    }
}
// ----
// TypeError 9231: (53-100): Library functions must be implemented if declared.
