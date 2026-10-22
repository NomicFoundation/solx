//! { "modes": [ "E" ], "cases": [ {
//!     "name": "default",
//!     "inputs": [
//!         {
//!             "method": "copy",
//!             "calldata": [
//!                 "0x20",
//!                 "5",
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
    struct S {
        uint256 a;
        bytes b;
    }

    function copy(S calldata s) external pure returns (uint256) {
        S memory m = s;
        return m.b.length;
    }
}
