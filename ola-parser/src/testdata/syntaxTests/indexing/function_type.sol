contract C {
  fn f()  {
    f[0];
  }
}
// ----
// TypeError 2614: (41-42): Indexed expression has to be a type, mapping or array (is fn ())
