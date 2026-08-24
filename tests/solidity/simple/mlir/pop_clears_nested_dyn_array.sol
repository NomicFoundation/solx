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
    uint256[][] arr;

    function test() public {
        arr.push();
        arr[0].push(111);
        arr[0].push(222);

        arr.pop();

        assembly {
            mstore(0, arr.slot)
            let outerData := keccak256(0, 0x20)
            // Inner array length slot must be zero.
            if sload(outerData) { revert(0, 0) }
            // Inner data slots must be zero.
            mstore(0, outerData)
            let innerData := keccak256(0, 0x20)
            if sload(innerData) { revert(0, 0) }
            if sload(add(innerData, 1)) { revert(0, 0) }
        }
        assert(arr.length == 0);
    }
}
