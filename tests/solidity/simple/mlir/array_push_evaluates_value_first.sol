//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "full()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "packed()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256[] a;
    uint8[] b;

    function full() external returns (uint256) {
        a.push(a.length);
        a.push(a.length);
        return a[0] * 10 + a[1];
    }

    function packed() external returns (uint256) {
        b.push(uint8(b.length));
        b.push(uint8(b.length));
        return uint256(b[0]) * 10 + b[1];
    }
}
