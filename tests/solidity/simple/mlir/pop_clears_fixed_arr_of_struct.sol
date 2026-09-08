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
    struct Inner {
        uint256 v;
        uint256[] data;
    }

    Inner[2][] arr;

    function test() public {
        arr.push();
        arr[0][0].v = 1;
        arr[0][0].data.push(10);
        arr[0][1].v = 2;
        arr[0][1].data.push(20);
        arr[0][1].data.push(30);
        arr.pop();
        assembly {
            mstore(0, arr.slot)
            let base := keccak256(0, 0x20)
            if sload(base)           { revert(0, 0) }  // Inner[0].v
            if sload(add(base, 1))   { revert(0, 0) }  // Inner[0].data length
            if sload(add(base, 2))   { revert(0, 0) }  // Inner[1].v
            if sload(add(base, 3))   { revert(0, 0) }  // Inner[1].data length
            mstore(0, add(base, 1))
            if sload(keccak256(0, 0x20)) { revert(0, 0) }  // Inner[0].data[0] = 10
            mstore(0, add(base, 3))
            let d1 := keccak256(0, 0x20)
            if sload(d1)             { revert(0, 0) }  // Inner[1].data[0] = 20
            if sload(add(d1, 1))     { revert(0, 0) }  // Inner[1].data[1] = 30
        }
        assert(arr.length == 0);
    }
}
