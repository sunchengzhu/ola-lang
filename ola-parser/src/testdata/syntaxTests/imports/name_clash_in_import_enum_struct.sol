==== Source: a ====
enum E { A }
==== Source: b ====
import "a";
struct E { u256 a; }
// ----
// DeclarationError 2333: (b:12-35): Identifier already declared.
