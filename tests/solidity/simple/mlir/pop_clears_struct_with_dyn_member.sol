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
        uint256[] data;
    }
    S[] arr;

    function test() public {
        arr.push();
        arr[0].x = 42;
        arr[0].data.push(100);
        arr[0].data.push(200);

        arr.pop(); // must zero x, data.length, data[0], data[1]

        assembly {
            mstore(0, arr.slot)
            let outerData := keccak256(0, 0x20)
            // S.x must be zero.
            if sload(outerData) { revert(0, 0) }
            let dataLenSlot := add(outerData, 1)
            // S.data.length must be zero.
            if sload(dataLenSlot) { revert(0, 0) }
            // S.data elements must be zero.
            mstore(0, dataLenSlot)
            let innerData := keccak256(0, 0x20)
            if sload(innerData) { revert(0, 0) }
            if sload(add(innerData, 1)) { revert(0, 0) }
        }
        assert(arr.length == 0);
    }
}
