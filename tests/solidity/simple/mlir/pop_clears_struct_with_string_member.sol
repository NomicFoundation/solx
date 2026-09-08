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
        string name;
    }
    S[] arr;

    function test() public {
        arr.push();
        arr[0].x = 7;
        arr[0].name = "this is a long enough name to be out-of-place storage";

        // arr[0].x     at outerData+0
        // arr[0].name  at outerData+1 (length slot)
        // arr[0].name data at keccak256(outerData+1)

        arr.pop();

        assembly {
            mstore(0, arr.slot)
            let outerData := keccak256(0, 0x20)
            if sload(outerData) { revert(0, 0) }
            let nameSlot := add(outerData, 1)
            if sload(nameSlot) { revert(0, 0) }
            mstore(0, nameSlot)
            if sload(keccak256(0, 0x20)) { revert(0, 0) }
        }
        assert(arr.length == 0);
    }
}
