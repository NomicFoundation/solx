//! { "modes": [ "E" ], "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "copy",
//!             "calldata": [
//!                 "0x20",
//!                 "1",
//!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [],
//!         "exception": true
//!     }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function copy(bytes[] calldata a) external pure returns (uint256) {
        bytes[] memory m = a;
        return m[0].length;
    }
}
