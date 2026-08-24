//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "test()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "#storage_empty",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": []
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint8[] arr;

    function test() public {
        for (uint256 i = 0; i < 32; i++)
            arr.push(uint8(i + 1));

        arr.pop();

        assembly {
            mstore(0, arr.slot)
            let dataSlot := keccak256(0, 0x20)
            let slot0 := sload(dataSlot)
            // Top byte (bits 255..248) must be zero, element 31 cleared.
            if shr(248, slot0) { revert(0, 0) }
            // Bottom byte (bits 7..0) must equal 1, element 0 preserved.
            if iszero(eq(and(slot0, 0xff), 1)) { revert(0, 0) }
        }
        assert(arr.length == 31);
        assert(arr[0] == 1);
        assert(arr[30] == 31);
        while (arr.length > 0)
            arr.pop();
    }
}
