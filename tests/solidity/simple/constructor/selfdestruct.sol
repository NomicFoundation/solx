//! { "cases": [ {
//!     "name": "main",
//!     "inputs": [
//!         {
//!             "method": "#deployer",
//!             "calldata": [
//!             ],
//!             "expected": [
//!                 "Test.address"
//!             ]
//!         }
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    constructor() {
        selfdestruct(payable(msg.sender));
    }
}
