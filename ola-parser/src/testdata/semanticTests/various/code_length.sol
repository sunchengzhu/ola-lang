// SPDX-License-Identifier: GPL-3.0
contract C {
    u256 len1;
    u256 len2;
    constructor() {
        u256 mem_ptr_before;
        u256 mem_ptr_after;

        assembly {
            mem_ptr_before := mload(64)
        }

        len1 = address(0).code.length;

        assembly {
            mem_ptr_after := mload(64)
        }

        // To check that no memory was allocated and written.
        assert(mem_ptr_before == mem_ptr_after);

        len2 = address(this).code.length;

        // To check that no memory was allocated and written.
        assembly {
            mem_ptr_after := mload(64)
        }

        assert(mem_ptr_before == mem_ptr_after);

    }

    fn f()  -> (bool r1, bool r2) {
        u256 mem_ptr_before;
        u256 mem_ptr_after;

        assembly {
            mem_ptr_before := mload(64)
        }

        r1 = address(this).code.length > 50;

        assembly {
            mem_ptr_after := mload(64)
        }

        // To check that no memory was allocated and written.
        assert(mem_ptr_before == mem_ptr_after);

        address a = address(0);
        r2 = a.code.length == 0;

        // To check that no memory was allocated and written.
        assembly {
            mem_ptr_after := mload(64)
        }

    }
}
// ====
// compileToEwasm: also
// compileViaYul: also
// ----
// constructor()
// gas legacy: 126455
// f(): true, true -> true, true