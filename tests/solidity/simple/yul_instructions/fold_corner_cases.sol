//! { "cases": [ {
//!     "name": "add_wrap",
//!     "inputs": [ { "method": "add_wrap", "calldata": [] } ],
//!     "expected": [ "0" ]
//! }, {
//!     "name": "add_wrap_max",
//!     "inputs": [ { "method": "add_wrap_max", "calldata": [] } ],
//!     "expected": [ "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe" ]
//! }, {
//!     "name": "sub_wrap",
//!     "inputs": [ { "method": "sub_wrap", "calldata": [] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "mul_wrap",
//!     "inputs": [ { "method": "mul_wrap", "calldata": [] } ],
//!     "expected": [ "0" ]
//! }, {
//!     "name": "mul_wrap_max",
//!     "inputs": [ { "method": "mul_wrap_max", "calldata": [] } ],
//!     "expected": [ "1" ]
//! }, {
//!     "name": "sdiv_neg_divisor",
//!     "inputs": [ { "method": "sdiv_neg_divisor", "calldata": [] } ],
//!     "expected": [ "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd" ]
//! }, {
//!     "name": "sdiv_neg_neg",
//!     "inputs": [ { "method": "sdiv_neg_neg", "calldata": [] } ],
//!     "expected": [ "3" ]
//! }, {
//!     "name": "smod_neg_divisor",
//!     "inputs": [ { "method": "smod_neg_divisor", "calldata": [] } ],
//!     "expected": [ "1" ]
//! }, {
//!     "name": "smod_neg_dividend",
//!     "inputs": [ { "method": "smod_neg_dividend", "calldata": [] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "exp_zero_zero",
//!     "inputs": [ { "method": "exp_zero_zero", "calldata": [] } ],
//!     "expected": [ "1" ]
//! }, {
//!     "name": "exp_wrap",
//!     "inputs": [ { "method": "exp_wrap", "calldata": [] } ],
//!     "expected": [ "0" ]
//! }, {
//!     "name": "exp_max_exponent",
//!     "inputs": [ { "method": "exp_max_exponent", "calldata": [] } ],
//!     "expected": [ "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab" ]
//! }, {
//!     "name": "signextend_byte",
//!     "inputs": [ { "method": "signextend_byte", "calldata": [] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80" ]
//! }, {
//!     "name": "signextend_high_bits_ignored",
//!     "inputs": [ { "method": "signextend_high_bits_ignored", "calldata": [] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "signextend_whole_word",
//!     "inputs": [ { "method": "signextend_whole_word", "calldata": [] } ],
//!     "expected": [ "0x8000000000000000000000000000000000000000000000000000000000000000" ]
//! }, {
//!     "name": "signextend_out_of_range",
//!     "inputs": [ { "method": "signextend_out_of_range", "calldata": [] } ],
//!     "expected": [ "0x80" ]
//! }, {
//!     "name": "shl_drain",
//!     "inputs": [ { "method": "shl_drain", "calldata": [] } ],
//!     "expected": [ "0" ]
//! }, {
//!     "name": "shr_drain",
//!     "inputs": [ { "method": "shr_drain", "calldata": [] } ],
//!     "expected": [ "0" ]
//! }, {
//!     "name": "sar_drain_negative",
//!     "inputs": [ { "method": "sar_drain_negative", "calldata": [] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "sar_drain_positive",
//!     "inputs": [ { "method": "sar_drain_positive", "calldata": [] } ],
//!     "expected": [ "0" ]
//! }, {
//!     "name": "sar_sign_fill",
//!     "inputs": [ { "method": "sar_sign_fill", "calldata": [] } ],
//!     "expected": [ "0xc000000000000000000000000000000000000000000000000000000000000000" ]
//! }, {
//!     "name": "addmod_no_wrap",
//!     "inputs": [ { "method": "addmod_no_wrap", "calldata": [] } ],
//!     "expected": [ "2" ]
//! }, {
//!     "name": "addmod_zero",
//!     "inputs": [ { "method": "addmod_zero", "calldata": [] } ],
//!     "expected": [ "0" ]
//! }, {
//!     "name": "mulmod_no_wrap",
//!     "inputs": [ { "method": "mulmod_no_wrap", "calldata": [] } ],
//!     "expected": [ "9" ]
//! }, {
//!     "name": "mulmod_zero",
//!     "inputs": [ { "method": "mulmod_zero", "calldata": [] } ],
//!     "expected": [ "0" ]
//! }, {
//!     "name": "slt_negative",
//!     "inputs": [ { "method": "slt_negative", "calldata": [] } ],
//!     "expected": [ "1" ]
//! }, {
//!     "name": "lt_negative",
//!     "inputs": [ { "method": "lt_negative", "calldata": [] } ],
//!     "expected": [ "0" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

// Every method evaluates one corner case on literals, which the compiler folds.
contract Test {
    function add_wrap() external pure returns (uint256 result) {
        assembly {
            result := add(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 1)
        }
    }

    function add_wrap_max() external pure returns (uint256 result) {
        assembly {
            result := add(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
        }
    }

    function sub_wrap() external pure returns (uint256 result) {
        assembly {
            result := sub(0, 1)
        }
    }

    function mul_wrap() external pure returns (uint256 result) {
        assembly {
            result := mul(0x8000000000000000000000000000000000000000000000000000000000000000, 2)
        }
    }

    function mul_wrap_max() external pure returns (uint256 result) {
        assembly {
            result := mul(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
        }
    }

    function sdiv_neg_divisor() external pure returns (uint256 result) {
        assembly {
            result := sdiv(7, 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe)
        }
    }

    function sdiv_neg_neg() external pure returns (uint256 result) {
        assembly {
            result := sdiv(0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff9, 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe)
        }
    }

    function smod_neg_divisor() external pure returns (uint256 result) {
        assembly {
            result := smod(7, 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe)
        }
    }

    function smod_neg_dividend() external pure returns (uint256 result) {
        assembly {
            result := smod(0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff9, 2)
        }
    }

    function exp_zero_zero() external pure returns (uint256 result) {
        assembly {
            result := exp(0, 0)
        }
    }

    function exp_wrap() external pure returns (uint256 result) {
        assembly {
            result := exp(2, 256)
        }
    }

    function exp_max_exponent() external pure returns (uint256 result) {
        assembly {
            result := exp(3, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
        }
    }

    function signextend_byte() external pure returns (uint256 result) {
        assembly {
            result := signextend(0, 0x80)
        }
    }

    function signextend_high_bits_ignored() external pure returns (uint256 result) {
        assembly {
            result := signextend(0, 0x1ff)
        }
    }

    function signextend_whole_word() external pure returns (uint256 result) {
        assembly {
            result := signextend(31, 0x8000000000000000000000000000000000000000000000000000000000000000)
        }
    }

    function signextend_out_of_range() external pure returns (uint256 result) {
        assembly {
            result := signextend(0x100000000000000000000000000000000000000000000000000, 0x80)
        }
    }

    function shl_drain() external pure returns (uint256 result) {
        assembly {
            result := shl(0x100000000000000000000000000000000000000000000000000, 1)
        }
    }

    function shr_drain() external pure returns (uint256 result) {
        assembly {
            result := shr(256, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
        }
    }

    function sar_drain_negative() external pure returns (uint256 result) {
        assembly {
            result := sar(0x100000000000000000000000000000000000000000000000000, 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffb)
        }
    }

    function sar_drain_positive() external pure returns (uint256 result) {
        assembly {
            result := sar(256, 5)
        }
    }

    function sar_sign_fill() external pure returns (uint256 result) {
        assembly {
            result := sar(1, 0x8000000000000000000000000000000000000000000000000000000000000000)
        }
    }

    function addmod_no_wrap() external pure returns (uint256 result) {
        assembly {
            result := addmod(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 7)
        }
    }

    function addmod_zero() external pure returns (uint256 result) {
        assembly {
            result := addmod(5, 6, 0)
        }
    }

    function mulmod_no_wrap() external pure returns (uint256 result) {
        assembly {
            result := mulmod(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 11)
        }
    }

    function mulmod_zero() external pure returns (uint256 result) {
        assembly {
            result := mulmod(5, 6, 0)
        }
    }

    function slt_negative() external pure returns (uint256 result) {
        assembly {
            result := slt(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 0)
        }
    }

    function lt_negative() external pure returns (uint256 result) {
        assembly {
            result := lt(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 0)
        }
    }
}
