//! { "cases": [ {
//!     "name": "zero_zero",
//!     "inputs": [
//!         {
//!             "method": "zero_zero",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "one_zero",
//!     "inputs": [
//!         {
//!             "method": "one_zero",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "max_zero",
//!     "inputs": [
//!         {
//!             "method": "max_zero",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "min_zero",
//!     "inputs": [
//!         {
//!             "method": "min_zero",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary",
//!     "inputs": [
//!         {
//!             "method": "ordinary",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function zero_zero() external pure returns (uint256 result) {
        assembly {
            result := mod(
                0x0000000000000000000000000000000000000000000000000000000000000000,
                0x0000000000000000000000000000000000000000000000000000000000000000
            )
        }
    }

    function one_zero() external pure returns (uint256 result) {
        assembly {
            result := mod(
                0x0000000000000000000000000000000000000000000000000000000000000001,
                0x0000000000000000000000000000000000000000000000000000000000000000
            )
        }
    }

    function max_zero() external pure returns (uint256 result) {
        assembly {
            result := mod(
                0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
                0x0000000000000000000000000000000000000000000000000000000000000000
            )
        }
    }

    function min_zero() external pure returns (uint256 result) {
        assembly {
            result := mod(
                0x8000000000000000000000000000000000000000000000000000000000000000,
                0x0000000000000000000000000000000000000000000000000000000000000000
            )
        }
    }

    function ordinary() external pure returns (uint256 result) {
        assembly {
            result := mod(10, 3)
        }
    }
}
