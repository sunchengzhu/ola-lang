library L {
  fn f()  {
    (super);
  }
}
// ----
// DeclarationError 7576: (41-46): Undeclared identifier. "super" is not (or not yet) visible at this point.
