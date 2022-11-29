contract C {
    fn testFunction1() public {}
    fn testFunction2() public {}
    fn testFunction3() public {}


    fn() external [] externalArray0;
    fn() external [] externalArray1;

    fn() internal [] internalArray0;
    fn() internal [] internalArray1;

    constructor() {
        externalArray0 = new fn() external[] (3);
        externalArray1 = [
            this.testFunction1,
            this.testFunction2,
            this.testFunction3
        ];

        internalArray0 = new fn() internal[] (3);
        internalArray1 = [
            testFunction1,
            testFunction2,
            testFunction3
        ];
    }

    fn copyExternalStorageArrayOfFunctionType() external returns (bool) {
        assert(keccak256(abi.encode(externalArray0)) != keccak256(abi.encode(externalArray1)));
        externalArray0 = externalArray1;
        return keccak256(abi.encode(externalArray0)) == keccak256(abi.encode(externalArray1));
    }

    fn copyInternalArrayOfFunctionType() external returns (bool) {
        internalArray0 = internalArray1;
        assert(internalArray0.length == 3);

        return
            internalArray0.length == internalArray1.length &&
            internalArray0[0] == internalArray1[0] &&
            internalArray0[1] == internalArray1[1] &&
            internalArray0[2] == internalArray1[2];
    }
}
// ====
// compileViaYul: also
// ----
// copyExternalStorageArrayOfFunctionType() -> true
// gas irOptimized: 104659
// gas legacy: 108722
// gas legacyOptimized: 102438
// copyInternalArrayOfFunctionType() -> true