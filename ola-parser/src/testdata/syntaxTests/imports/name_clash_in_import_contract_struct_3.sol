==== Source: a ====
contract A {}
==== Source: b ====
import {A as b} from "a";
struct b { u256 a; }
// ----
// DeclarationError 2333: (b:26-49): Identifier already declared.
