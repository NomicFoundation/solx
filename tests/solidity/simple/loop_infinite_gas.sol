//! { "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "run",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [ {
//!         "return_data": [
//!         ],
//!         "exception": true
//!     } ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.20;

contract Test {
    uint256 public sink;

    function run() external returns (uint256) {
        uint256 i = 0;
        while (true) {
            i += 1;
        }
        return 42;
    }
}
