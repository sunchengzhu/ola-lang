contract Lotto {
    u256 const ticketPrice = 555;
}
// ====
// compileToEwasm: also
// compileViaYul: also
// ----
// ticketPrice() -> 555
