==== Source: a ====
struct A { u256 a; }
==== Source: b ====
import {A} from "a";
struct A { u256 a; }
// ----
// DeclarationError 2333: (b:21-44): Identifier already declared.