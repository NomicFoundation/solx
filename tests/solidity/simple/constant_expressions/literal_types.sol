//! { "cases": [ {
//!     "name": "neg_min_int8",
//!     "inputs": [
//!         {
//!             "method": "neg_min_int8",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-128"
//!     ]
//! }, {
//!     "name": "neg_one_int8",
//!     "inputs": [
//!         {
//!             "method": "neg_one_int8",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "-1"
//!     ]
//! }, {
//!     "name": "bool_false",
//!     "inputs": [
//!         {
//!             "method": "bool_false",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "zero_literal",
//!     "inputs": [
//!         {
//!             "method": "zero_literal",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "bool_true",
//!     "inputs": [
//!         {
//!             "method": "bool_true",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "hex_underscored",
//!     "inputs": [
//!         {
//!             "method": "hex_underscored",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "239"
//!     ]
//! }, {
//!     "name": "address_literal",
//!     "inputs": [
//!         {
//!             "method": "address_literal",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x00000000000000000000000000000000000000ff"
//!     ]
//! }, {
//!     "name": "time_units",
//!     "inputs": [
//!         {
//!             "method": "time_units",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "86400"
//!     ]
//! }, {
//!     "name": "digit_separators",
//!     "inputs": [
//!         {
//!             "method": "digit_separators",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "2000000"
//!     ]
//! }, {
//!     "name": "ether_unit",
//!     "inputs": [
//!         {
//!             "method": "ether_unit",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1000000000000000000"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function neg_min_int8() public pure returns (int8) {
        return -128;
    }

    function neg_one_int8() public pure returns (int8) {
        return -1;
    }

    function bool_false() public pure returns (bool) {
        return false;
    }

    function zero_literal() public pure returns (uint256) {
        return 0;
    }

    function bool_true() public pure returns (bool) {
        return true;
    }

    function hex_underscored() public pure returns (uint256) {
        return 0xde_ad_be_ef & 0xff;
    }

    function address_literal() public pure returns (address) {
        return 0x00000000000000000000000000000000000000ff;
    }

    function time_units() public pure returns (uint256) {
        return 1 days;
    }

    function digit_separators() public pure returns (uint256) {
        return 1_000_000 * 2;
    }

    function ether_unit() public pure returns (uint256) {
        return 1 ether;
    }
}
