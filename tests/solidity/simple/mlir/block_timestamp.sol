//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "#deployer",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "comment": "This is the 1st block",
//!                     "expected": [
//!                         "Test.address"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "comment": "This is the 2nd block (each block is \"15 seconds\")",
//!                     "expected": [
//!                         "0x1e"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "comment": "This is the 3rd block",
//!                     "expected": [
//!                         "0x2d"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    constructor() {}
    function f() public returns (uint) {
        return block.timestamp;
    }
}
