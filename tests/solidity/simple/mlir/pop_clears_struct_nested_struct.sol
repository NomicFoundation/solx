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
        bool active;
    }
    struct Outer {
        uint256 a;
        Inner inner;
        uint256 b;
    }
    Outer[] arr;

    function test() public {
        arr.push();
        arr[0].a = 1;
        arr[0].inner.v = 2;
        arr[0].inner.active = true;
        arr[0].b = 3;

        arr.pop();

        assembly {
            mstore(0, arr.slot)
            let base := keccak256(0, 0x20)
            if sload(base)           { revert(0, 0) }  // a
            if sload(add(base, 1))   { revert(0, 0) }  // inner.v
            if sload(add(base, 2))   { revert(0, 0) }  // inner.active
            if sload(add(base, 3))   { revert(0, 0) }  // b
        }
        assert(arr.length == 0);
    }
}
