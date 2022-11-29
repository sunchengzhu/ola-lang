contract c {
    uint[9] m_data;
    uint[] m_data_dyn;
    uint8[][] m_byte_data;
    fn store(uint[9] calldata a, uint8[3][] calldata b) external -> (uint8) {
        m_data = a;
        m_data_dyn = a;
        m_byte_data = b;
        return b[3][1]; // note that access and declaration are reversed to each other
    }
    fn retrieve() public -> (uint a, uint b, uint c, uint d, uint e, uint f, uint g) {
        a = m_data.length;
        b = m_data[7];
        c = m_data_dyn.length;
        d = m_data_dyn[7];
        e = m_byte_data.length;
        f = m_byte_data[3].length;
        g = m_byte_data[3][1];
    }
}
// ====
// compileViaYul: also
// ----
// store(u256[9],uint8[3][]): 21, 22, 23, 24, 25, 26, 27, 28, 29, 0x140, 4, 1, 2, 3, 11, 12, 13, 21, 22, 23, 31, 32, 33 -> 32
// gas irOptimized: 650608
// gas legacy: 694515
// gas legacyOptimized: 694013
// retrieve() -> 9, 28, 9, 28, 4, 3, 32