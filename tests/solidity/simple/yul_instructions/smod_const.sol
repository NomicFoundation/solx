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
//!         "0"
//!     ]
//! }, {
//!     "name": "min_by_minus_one_switch",
//!     "inputs": [
//!         {
//!             "method": "min_by_minus_one_switch",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "min_by_minus_one_add",
//!     "inputs": [
//!         {
//!             "method": "min_by_minus_one_add",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "5"
//!     ]
//! }, {
//!     "name": "min_by_minus_one_cmp",
//!     "inputs": [
//!         {
//!             "method": "min_by_minus_one_cmp",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "7"
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
    function zero_zero() external pure returns (int256 result) {
        assembly {
            result := smod(
                0x0000000000000000000000000000000000000000000000000000000000000000,
                0x0000000000000000000000000000000000000000000000000000000000000000
            )
        }
    }

    function one_zero() external pure returns (int256 result) {
        assembly {
            result := smod(
                0x0000000000000000000000000000000000000000000000000000000000000001,
                0x0000000000000000000000000000000000000000000000000000000000000000
            )
        }
    }

    function neg1_zero() external pure returns (int256 result) {
        assembly {
            result := smod(
                0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
                0x0000000000000000000000000000000000000000000000000000000000000000
            )
        }
    }

    function min_zero() external pure returns (int256 result) {
        assembly {
            result := smod(
                0x8000000000000000000000000000000000000000000000000000000000000000,
                0x0000000000000000000000000000000000000000000000000000000000000000
            )
        }
    }

    function min_by_minus_one() external pure returns (int256 result) {
        assembly {
            result := smod(
                0x8000000000000000000000000000000000000000000000000000000000000000,
                0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
            )
        }
    }

    function min_by_minus_one_switch() external pure returns (int256 result) {
        assembly {
            let r := smod(
                0x8000000000000000000000000000000000000000000000000000000000000000,
                0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
            )
            switch r
            case 0 {
                result := 1
            }
            default {
                result := 2
            }
        }
    }

    function min_by_minus_one_add() external pure returns (int256 result) {
        assembly {
            let r := smod(
                0x8000000000000000000000000000000000000000000000000000000000000000,
                0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
            )
            result := add(r, 5)
        }
    }

    function min_by_minus_one_cmp() external pure returns (int256 result) {
        assembly {
            let r := smod(
                0x8000000000000000000000000000000000000000000000000000000000000000,
                0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
            )
            switch lt(r, 0)
            case 1 {
                result := sub(0, 1)
            }
            default {
                result := 7
            }
        }
    }

    function ordinary() external pure returns (int256 result) {
        assembly {
            result := smod(10, 3)
        }
    }
}
