contract Foo {
	u256[] memory storage calldata x;
}
// ----
// ParserError 2314: (23-29): Expected identifier but got 'memory'
