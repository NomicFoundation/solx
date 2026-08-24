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
    uint72[] arr;

    function test() public {
        arr.push(0xAAAAAAAAAAAAAAAAA);
        arr.push(0xBBBBBBBBBBBBBBBBB);
        arr.push(0xCCCCCCCCCCCCCCCCC);

        arr.pop();

        assembly {
            mstore(0, arr.slot)
            let dataSlot := keccak256(0, 0x20)
            let slot0 := sload(dataSlot)
            // Bits 255..144 (element 2's region and above) must be zero.
            if shr(144, slot0) { revert(0, 0) }
            // Bits 71..0 (element 0) must be non-zero (preserved).
            if iszero(and(slot0, sub(shl(72, 1), 1))) { revert(0, 0) }
        }
        assert(arr.length == 2);
        assert(arr[0] == 0xAAAAAAAAAAAAAAAAA);
        assert(arr[1] == 0xBBBBBBBBBBBBBBBBB);
        arr.pop();
        arr.pop();
    }
}
