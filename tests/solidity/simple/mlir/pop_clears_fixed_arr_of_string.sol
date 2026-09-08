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
    string[2][] arr;

    function test() public {
        arr.push();
        string memory s0 = "this string is long enough to be stored out-of-place ok";
        string memory s1 = "second long string also out of place in the storage slot";
        arr[0][0] = s0;
        arr[0][1] = s1;
        arr.pop();
        assembly {
            mstore(0, arr.slot)
            let base := keccak256(0, 0x20)
            if sload(base)           { revert(0, 0) }  // arr[0][0] length slot
            if sload(add(base, 1))   { revert(0, 0) }  // arr[0][1] length slot
        }
        assert(arr.length == 0);
    }
}
