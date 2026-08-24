//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "200000000"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ],
//!     "ignore": true,
//!     "comment": "EVMVersion <paris cannot run on default EVM"
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function f() public returns (uint) {
        return block.difficulty;
    }
}
