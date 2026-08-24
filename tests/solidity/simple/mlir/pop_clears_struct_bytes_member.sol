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
    struct S {
        uint256 x;
        bytes data;
    }
    S[] arr;

    function test() public {
        arr.push();
        arr[0].x = 42;
        // out-of-place encoding: length > 31 bytes
        arr[0].data = hex"aabbccddee112233445566778899001122334455667788990011223344556677889900112233";

        arr.pop();

        assembly {
            mstore(0, arr.slot)
            let base := keccak256(0, 0x20)
            if sload(base)           { revert(0, 0) }  // x
            let dataSlot := add(base, 1)
            if sload(dataSlot)       { revert(0, 0) }  // data length slot
            mstore(0, dataSlot)
            if sload(keccak256(0, 0x20)) { revert(0, 0) }  // first data chunk
        }
        assert(arr.length == 0);
    }
}
