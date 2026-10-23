//! { "modes": [ "E" ], "cases": [ {
//!     "name": "bytes",
//!     "inputs": [
//!         {
//!             "method": "bytesLength",
//!             "calldata": [
//!                 "0x20",
//!                 "0x10000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000004100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! }, {
//!     "name": "array",
//!     "inputs": [
//!         {
//!             "method": "arrayLength",
//!             "calldata": [
//!                 "0x20",
//!                 "0x10000000000000000"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000004100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function bytesLength(bytes memory b) external pure returns (uint256) {
        return b.length;
    }

    function arrayLength(uint256[] memory a) external pure returns (uint256) {
        return a.length;
    }
}
