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
        arr.push(); arr[0].push(10); arr[0].push(20);
        arr.push(); arr[1].push(30); arr[1].push(40); arr[1].push(50);

        arr.pop(); // removes arr[1] (3 inner elements)

        assembly {
            mstore(0, arr.slot)
            let outerData := keccak256(0, 0x20)
            // arr[1].length slot (outerData+1) must be zero.
            let len1Slot := add(outerData, 1)
            if sload(len1Slot) { revert(0, 0) }
            // arr[1] inner data slots must all be zero.
            mstore(0, len1Slot)
            let innerData1 := keccak256(0, 0x20)
            if sload(innerData1) { revert(0, 0) }
            if sload(add(innerData1, 1)) { revert(0, 0) }
            if sload(add(innerData1, 2)) { revert(0, 0) }
        }
        assert(arr.length == 1);
        assert(arr[0].length == 2);
        assert(arr[0][0] == 10);
        assert(arr[0][1] == 20);
        arr.pop();
    }
}
