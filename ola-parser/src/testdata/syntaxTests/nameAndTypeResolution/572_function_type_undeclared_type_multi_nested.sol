contract C {
    fn a(fn(fn(fn(Nested)))) external pure {}
}
// ----
// DeclarationError 7920: (55-61): Identifier not found or not unique.