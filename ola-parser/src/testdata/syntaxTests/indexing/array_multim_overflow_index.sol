contract C {
  fn f()  {
    bytes[32] memory a;
    a[8**90][8**90][1 - 8**90];
  }
}
// ----
// TypeError 7407: (67-72): Type int_const 1897...(74 digits omitted)...1424 is not implicitly convertible to expected type u256. Literal is too large to fit in u256.
// TypeError 7407: (74-79): Type int_const 1897...(74 digits omitted)...1424 is not implicitly convertible to expected type u256. Literal is too large to fit in u256.
// TypeError 7407: (81-90): Type int_const -189...(75 digits omitted)...1423 is not implicitly convertible to expected type u256. Cannot implicitly convert signed literal to unsigned type.
// TypeError 6318: (65-91): Index expression cannot be represented as an unsigned integer.
