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
//!     "name": "neg1_zero",
//!     "inputs": [
//!         {
//!             "method": "neg1_zero",
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
//!     "name": "min_by_minus_one",
//!     "inputs": [
//!         {
//!             "method": "min_by_minus_one",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0x8000000000000000000000000000000000000000000000000000000000000000"
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
//!         "3"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function zero_zero() external pure returns (int256 result) {
        assembly {
            result := sdiv(
                0x0000000000000000000000000000000000000000000000000000000000000000,
                0x0000000000000000000000000000000000000000000000000000000000000000
            )
        }
    }

    function one_zero() external pure returns (int256 result) {
        assembly {
            result := sdiv(
                0x0000000000000000000000000000000000000000000000000000000000000001,
                0x0000000000000000000000000000000000000000000000000000000000000000
            )
        }
    }

    function neg1_zero() external pure returns (int256 result) {
        assembly {
            result := sdiv(
                0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
                0x0000000000000000000000000000000000000000000000000000000000000000
            )
        }
    }

    function min_zero() external pure returns (int256 result) {
        assembly {
            result := sdiv(
                0x8000000000000000000000000000000000000000000000000000000000000000,
                0x0000000000000000000000000000000000000000000000000000000000000000
            )
        }
    }

    function min_by_minus_one() external pure returns (int256 result) {
        assembly {
            result := sdiv(
                0x8000000000000000000000000000000000000000000000000000000000000000,
                0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
            )
        }
    }

    function ordinary() external pure returns (int256 result) {
        assembly {
            result := sdiv(10, 3)
        }
    }
}
