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
        uint256[][2] data;
        uint256 y;
    }

    S[] arr;

    function test() public {
        arr.push();
        arr[0].x = 42;
        arr[0].data[0].push(100);
        arr[0].data[1].push(200);
        arr[0].data[1].push(300);
        arr[0].y = 99;
        arr.pop();
        assembly {
            mstore(0, arr.slot)
            let base := keccak256(0, 0x20)
            if sload(base)           { revert(0, 0) }  // slot 0: x
            if sload(add(base, 1))   { revert(0, 0) }  // slot 1: data[0] length
            if sload(add(base, 2))   { revert(0, 0) }  // slot 2: data[1] length
            if sload(add(base, 3))   { revert(0, 0) }  // slot 3: y
            mstore(0, add(base, 1))
            if sload(keccak256(0, 0x20)) { revert(0, 0) }  // data[0][0] = 100
            mstore(0, add(base, 2))
            let d1 := keccak256(0, 0x20)
            if sload(d1)             { revert(0, 0) }  // data[1][0] = 200
            if sload(add(d1, 1))     { revert(0, 0) }  // data[1][1] = 300
        }
        assert(arr.length == 0);
    }
}
