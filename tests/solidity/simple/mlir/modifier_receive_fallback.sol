//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "#fallback",
//!                     "calldata": [],
//!                     "value": "1 wei",
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "#fallback",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": []
//!                 },
//!                 {
//!                     "method": "count()",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 public count;

    modifier counted() {
        count += 1;
        _;
        return;
    }

    receive() external payable counted {}

    fallback() external counted {}
}
