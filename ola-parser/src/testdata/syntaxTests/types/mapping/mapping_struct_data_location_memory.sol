pragma abicoder               v2;
contract C {
    struct S { mapping(uint => uint) a; }
    fn f(S memory) public {}
}
// ----
// TypeError 4103: (104-112): Types containing (nested) mappings can only be parameters or return variables of internal or library functions.