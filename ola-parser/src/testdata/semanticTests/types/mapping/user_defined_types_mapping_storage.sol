type A is uint;
type B is uint;

library L {
  fn f(mapping(A=>B) storage _m, B _v) public { _m[A.wrap(uint(2))] = _v; }
  fn f(mapping(uint=>uint) storage _m, uint _v) public { _m[uint(3)] = _v; }
}

contract C {
	mapping(uint=>uint) uintMap;
	mapping(A=>B) abMap;

	fn testAB() public returns (bool) {
		L.f(abMap, B.wrap(3));
		return B.unwrap(abMap[A.wrap(uint(2))]) == 3;
	}
	fn testUint() public returns (bool) {
		L.f(uintMap, 4);
		return uintMap[3] == 4;
	}
}

// ====
// compileViaYul: also
// ----
// library: L
// testAB() -> true
// testUint() -> true