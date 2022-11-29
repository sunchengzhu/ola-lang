contract c {
    u256 constant a1 = 0;
    u256 constant a2 = 1;
    u256 constant b1 = mulmod(3, 4, 0);
    u256 constant b2 = mulmod(3, 4, a1);
    u256 constant b3 = mulmod(3, 4, a2 - 1);
}
// ----
// TypeError 4195: (88-103): Arithmetic modulo zero.
// TypeError 4195: (128-144): Arithmetic modulo zero.
// TypeError 4195: (169-189): Arithmetic modulo zero.