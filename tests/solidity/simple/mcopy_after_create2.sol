//! { "cases": [ {
//!     "name": "mcopy_not_rewritten_to_returndatacopy",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.25;

// Regression test: a heap-to-heap MCOPY that follows a call-like instruction
// must not be rewritten into a copy from return data. Call-like instructions
// (here CREATE2) replace the return-data buffer, so after CREATE2 the bytes
// previously saved with RETURNDATACOPY exist only on the heap. The unsound
// rewrite
//
//     RETURNDATACOPY  heap <- return_data
//     CREATE2
//     MCOPY           heap <- heap
// into
//     RETURNDATACOPY  heap <- return_data
//     CREATE2
//     RETURNDATACOPY  heap <- return_data
//
// makes the final copy read out of bounds of the (now empty) return-data
// buffer and revert, whereas the correct code copies the saved heap bytes.
contract Test {
    function main() external returns (uint256) {
        assembly ("memory-safe") {
            mstore(0x40, 0x500)

            // Identity precompile: returns the 18 input bytes as return data.
            mstore(0x180, 0x112233445566778899aabbccddeeff0011220000000000000000000000000000)
            pop(call(100000, 0x04, 0, 0x180, 18, 0, 0))

            // Save the current return-data buffer into heap memory.
            returndatacopy(0x200, 0, 18)

            // CREATE2 clobbers the EVM return-data buffer.
            mstore(0x300, 0x6001600c60003960016000f30000000000000000000000000000000000000000)
            pop(create2(0, 0x300, 13, 0))

            // Heap-to-heap copy of the bytes saved before CREATE2. If this is
            // miscompiled into RETURNDATACOPY, it reverts: the return-data
            // buffer is empty after a successful CREATE2, so reading 7 bytes
            // from it is out of bounds.
            mcopy(0x101, 0x200, 7)

            // Verify the 7 copied bytes match the start of the saved data.
            mstore(0, eq(shr(200, mload(0x101)), 0x11223344556677))
            return(0, 32)
        }
    }
}
