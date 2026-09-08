//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "direct()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0xff"
//!                     ]
//!                 },
//!                 {
//!                     "method": "via_local()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0xff"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function direct() external pure returns (address) {
        return 0x00000000000000000000000000000000000000ff;
    }

    function via_local() external pure returns (address) {
        address a = 0x00000000000000000000000000000000000000ff;
        return a;
    }
}
