contract C {
  fn() Test;

  fn f()  {
    emit Test();
  }
}
// ----
// TypeError 9292: (66-70): Expression has to be an event invocation.
