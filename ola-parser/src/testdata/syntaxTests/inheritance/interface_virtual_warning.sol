interface I {
	fn foo() virtual external;
}
// ----
// Warning 5815: (15-47): Interface functions are implicitly "virtual"
