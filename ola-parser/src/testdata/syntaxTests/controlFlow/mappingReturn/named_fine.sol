contract C {
    mapping(uint=>uint) m;
    fn f() internal view returns (mapping(uint=>uint) storage r) { r = m; }
}
// ----