interface I {
  fn f() external m pure -> (u256);
  modifier m() { _; }
}
// ----
// SyntaxError 5842: (16-60): Functions in interfaces cannot have modifiers.
// TypeError 6408: (63-82): Modifiers cannot be defined or declared in interfaces.
