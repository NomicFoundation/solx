//! { "cases": [ {
//!     "name": "first_byte",
//!     "inputs": [
//!         {
//!             "method": "first_byte",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0x01"
//!     ]
//! }, {
//!     "name": "last_byte",
//!     "inputs": [
//!         {
//!             "method": "last_byte",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0x20"
//!     ]
//! }, {
//!     "name": "pow253",
//!     "inputs": [
//!         {
//!             "method": "pow253",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "pow253_plus_one",
//!     "inputs": [
//!         {
//!             "method": "pow253_plus_one",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "pow254",
//!     "inputs": [
//!         {
//!             "method": "pow254",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "three_pow253",
//!     "inputs": [
//!         {
//!             "method": "three_pow253",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function first_byte() external pure returns (uint256 result) {
        assembly {
            result := byte(
                0x0000000000000000000000000000000000000000000000000000000000000000,
                0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
            )
        }
    }

    function last_byte() external pure returns (uint256 result) {
        assembly {
            result := byte(
                0x000000000000000000000000000000000000000000000000000000000000001f,
                0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
            )
        }
    }

    function pow253() external pure returns (uint256 result) {
        assembly {
            result := byte(
                0x2000000000000000000000000000000000000000000000000000000000000000,
                0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
            )
        }
    }

    function pow253_plus_one() external pure returns (uint256 result) {
        assembly {
            result := byte(
                0x2000000000000000000000000000000000000000000000000000000000000001,
                0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
            )
        }
    }

    function pow254() external pure returns (uint256 result) {
        assembly {
            result := byte(
                0x4000000000000000000000000000000000000000000000000000000000000000,
                0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
            )
        }
    }

    function three_pow253() external pure returns (uint256 result) {
        assembly {
            result := byte(
                0x6000000000000000000000000000000000000000000000000000000000000000,
                0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
            )
        }
    }
}
