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
    uint256[] arr;

    function test() public {
        arr.push(111);
        arr.push(222);
        arr.push(333);

        arr.pop(); // removes element at index 2

        // Slot at index 2 must be zero after pop.
        assembly {
            mstore(0, arr.slot)
            let dataSlot := keccak256(0, 0x20)
            if sload(add(dataSlot, 2)) { revert(0, 0) }
        }
        assert(arr.length == 2);
        assert(arr[0] == 111);
        assert(arr[1] == 222);
        arr.pop();
        arr.pop();
    }
}
