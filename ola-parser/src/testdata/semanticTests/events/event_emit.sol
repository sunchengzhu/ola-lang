contract ClientReceipt {
    event Deposit(address indexed _from, bytes32 indexed _id, uint _value);
    fn deposit(bytes32 _id) public payable {
        emit Deposit(msg.sender, _id, msg.value);
    }
}
// ====
// compileViaYul: also
// ----
// deposit(bytes32), 18 wei: 0x1234 ->
// ~ emit Deposit(address,bytes32,u256): #0x1212121212121212121212121212120000000012, #0x1234, 0x12