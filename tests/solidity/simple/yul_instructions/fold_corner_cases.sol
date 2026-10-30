//! { "cases": [ {
//!     "name": "add_wrap",
//!     "inputs": [ { "method": "add_wrap", "calldata": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "1" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "add_wrap_max",
//!     "inputs": [ { "method": "add_wrap_max", "calldata": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ] } ],
//!     "expected": [ "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe", "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe" ]
//! }, {
//!     "name": "sub_wrap",
//!     "inputs": [ { "method": "sub_wrap", "calldata": [ "0", "1" ] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "mul_wrap",
//!     "inputs": [ { "method": "mul_wrap", "calldata": [ "0x8000000000000000000000000000000000000000000000000000000000000000", "2" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "mul_wrap_max",
//!     "inputs": [ { "method": "mul_wrap_max", "calldata": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ] } ],
//!     "expected": [ "1", "1" ]
//! }, {
//!     "name": "div_zero",
//!     "inputs": [ { "method": "div_zero", "calldata": [ "7", "0" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "sdiv_zero",
//!     "inputs": [ { "method": "sdiv_zero", "calldata": [ "7", "0" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "sdiv_min_neg_one",
//!     "inputs": [ { "method": "sdiv_min_neg_one", "calldata": [ "0x8000000000000000000000000000000000000000000000000000000000000000", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ] } ],
//!     "expected": [ "0x8000000000000000000000000000000000000000000000000000000000000000", "0x8000000000000000000000000000000000000000000000000000000000000000" ]
//! }, {
//!     "name": "sdiv_neg_divisor",
//!     "inputs": [ { "method": "sdiv_neg_divisor", "calldata": [ "7", "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe" ] } ],
//!     "expected": [ "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd", "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd" ]
//! }, {
//!     "name": "sdiv_neg_neg",
//!     "inputs": [ { "method": "sdiv_neg_neg", "calldata": [ "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff9", "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe" ] } ],
//!     "expected": [ "3", "3" ]
//! }, {
//!     "name": "mod_zero",
//!     "inputs": [ { "method": "mod_zero", "calldata": [ "7", "0" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "smod_zero",
//!     "inputs": [ { "method": "smod_zero", "calldata": [ "7", "0" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "smod_neg_divisor",
//!     "inputs": [ { "method": "smod_neg_divisor", "calldata": [ "7", "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe" ] } ],
//!     "expected": [ "1", "1" ]
//! }, {
//!     "name": "smod_neg_dividend",
//!     "inputs": [ { "method": "smod_neg_dividend", "calldata": [ "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff9", "2" ] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "smod_min_neg_one",
//!     "inputs": [ { "method": "smod_min_neg_one", "calldata": [ "0x8000000000000000000000000000000000000000000000000000000000000000", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "exp_zero_zero",
//!     "inputs": [ { "method": "exp_zero_zero", "calldata": [ "0", "0" ] } ],
//!     "expected": [ "1", "1" ]
//! }, {
//!     "name": "exp_wrap",
//!     "inputs": [ { "method": "exp_wrap", "calldata": [ "2", "256" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "exp_max_exponent",
//!     "inputs": [ { "method": "exp_max_exponent", "calldata": [ "3", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ] } ],
//!     "expected": [ "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab", "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab" ]
//! }, {
//!     "name": "signextend_byte",
//!     "inputs": [ { "method": "signextend_byte", "calldata": [ "0", "0x80" ] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80" ]
//! }, {
//!     "name": "signextend_high_bits_ignored",
//!     "inputs": [ { "method": "signextend_high_bits_ignored", "calldata": [ "0", "0x1ff" ] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "signextend_whole_word",
//!     "inputs": [ { "method": "signextend_whole_word", "calldata": [ "31", "0x8000000000000000000000000000000000000000000000000000000000000000" ] } ],
//!     "expected": [ "0x8000000000000000000000000000000000000000000000000000000000000000", "0x8000000000000000000000000000000000000000000000000000000000000000" ]
//! }, {
//!     "name": "signextend_out_of_range",
//!     "inputs": [ { "method": "signextend_out_of_range", "calldata": [ "0x100000000000000000000000000000000000000000000000000", "0x80" ] } ],
//!     "expected": [ "0x80", "0x80" ]
//! }, {
//!     "name": "byte_last",
//!     "inputs": [ { "method": "byte_last", "calldata": [ "31", "0x1234" ] } ],
//!     "expected": [ "0x34", "0x34" ]
//! }, {
//!     "name": "byte_out_of_range",
//!     "inputs": [ { "method": "byte_out_of_range", "calldata": [ "0x100000000000000000000000000000000000000000000000000", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "shl_drain",
//!     "inputs": [ { "method": "shl_drain", "calldata": [ "0x100000000000000000000000000000000000000000000000000", "1" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "shr_drain",
//!     "inputs": [ { "method": "shr_drain", "calldata": [ "256", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "sar_drain_negative",
//!     "inputs": [ { "method": "sar_drain_negative", "calldata": [ "0x100000000000000000000000000000000000000000000000000", "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffb" ] } ],
//!     "expected": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" ]
//! }, {
//!     "name": "sar_drain_positive",
//!     "inputs": [ { "method": "sar_drain_positive", "calldata": [ "256", "5" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "sar_sign_fill",
//!     "inputs": [ { "method": "sar_sign_fill", "calldata": [ "1", "0x8000000000000000000000000000000000000000000000000000000000000000" ] } ],
//!     "expected": [ "0xc000000000000000000000000000000000000000000000000000000000000000", "0xc000000000000000000000000000000000000000000000000000000000000000" ]
//! }, {
//!     "name": "addmod_no_wrap",
//!     "inputs": [ { "method": "addmod_no_wrap", "calldata": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "7" ] } ],
//!     "expected": [ "2", "2" ]
//! }, {
//!     "name": "addmod_zero",
//!     "inputs": [ { "method": "addmod_zero", "calldata": [ "5", "6", "0" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "mulmod_no_wrap",
//!     "inputs": [ { "method": "mulmod_no_wrap", "calldata": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "11" ] } ],
//!     "expected": [ "9", "9" ]
//! }, {
//!     "name": "mulmod_zero",
//!     "inputs": [ { "method": "mulmod_zero", "calldata": [ "5", "6", "0" ] } ],
//!     "expected": [ "0", "0" ]
//! }, {
//!     "name": "slt_negative",
//!     "inputs": [ { "method": "slt_negative", "calldata": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0" ] } ],
//!     "expected": [ "1", "1" ]
//! }, {
//!     "name": "lt_negative",
//!     "inputs": [ { "method": "lt_negative", "calldata": [ "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0" ] } ],
//!     "expected": [ "0", "0" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

// Every method evaluates one corner case twice: once on its calldata, which
// the compiler cannot fold, and once on the same values as literals, which it
// folds.
contract Test {
    function add_wrap(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := add(a, b)
            folded := add(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 1)
        }
    }

    function add_wrap_max(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := add(a, b)
            folded := add(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
        }
    }

    function sub_wrap(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := sub(a, b)
            folded := sub(0, 1)
        }
    }

    function mul_wrap(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := mul(a, b)
            folded := mul(0x8000000000000000000000000000000000000000000000000000000000000000, 2)
        }
    }

    function mul_wrap_max(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := mul(a, b)
            folded := mul(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
        }
    }

    function div_zero(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := div(a, b)
            folded := div(7, 0)
        }
    }

    function sdiv_zero(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := sdiv(a, b)
            folded := sdiv(7, 0)
        }
    }

    function sdiv_min_neg_one(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := sdiv(a, b)
            folded := sdiv(0x8000000000000000000000000000000000000000000000000000000000000000, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
        }
    }

    function sdiv_neg_divisor(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := sdiv(a, b)
            folded := sdiv(7, 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe)
        }
    }

    function sdiv_neg_neg(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := sdiv(a, b)
            folded := sdiv(0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff9, 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe)
        }
    }

    function mod_zero(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := mod(a, b)
            folded := mod(7, 0)
        }
    }

    function smod_zero(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := smod(a, b)
            folded := smod(7, 0)
        }
    }

    function smod_neg_divisor(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := smod(a, b)
            folded := smod(7, 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe)
        }
    }

    function smod_neg_dividend(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := smod(a, b)
            folded := smod(0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff9, 2)
        }
    }

    function smod_min_neg_one(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := smod(a, b)
            folded := smod(0x8000000000000000000000000000000000000000000000000000000000000000, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
        }
    }

    function exp_zero_zero(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := exp(a, b)
            folded := exp(0, 0)
        }
    }

    function exp_wrap(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := exp(a, b)
            folded := exp(2, 256)
        }
    }

    function exp_max_exponent(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := exp(a, b)
            folded := exp(3, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
        }
    }

    function signextend_byte(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := signextend(a, b)
            folded := signextend(0, 0x80)
        }
    }

    function signextend_high_bits_ignored(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := signextend(a, b)
            folded := signextend(0, 0x1ff)
        }
    }

    function signextend_whole_word(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := signextend(a, b)
            folded := signextend(31, 0x8000000000000000000000000000000000000000000000000000000000000000)
        }
    }

    function signextend_out_of_range(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := signextend(a, b)
            folded := signextend(0x100000000000000000000000000000000000000000000000000, 0x80)
        }
    }

    function byte_last(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := byte(a, b)
            folded := byte(31, 0x1234)
        }
    }

    function byte_out_of_range(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := byte(a, b)
            folded := byte(0x100000000000000000000000000000000000000000000000000, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
        }
    }

    function shl_drain(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := shl(a, b)
            folded := shl(0x100000000000000000000000000000000000000000000000000, 1)
        }
    }

    function shr_drain(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := shr(a, b)
            folded := shr(256, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
        }
    }

    function sar_drain_negative(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := sar(a, b)
            folded := sar(0x100000000000000000000000000000000000000000000000000, 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffb)
        }
    }

    function sar_drain_positive(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := sar(a, b)
            folded := sar(256, 5)
        }
    }

    function sar_sign_fill(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := sar(a, b)
            folded := sar(1, 0x8000000000000000000000000000000000000000000000000000000000000000)
        }
    }

    function addmod_no_wrap(uint256 a, uint256 b, uint256 c) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := addmod(a, b, c)
            folded := addmod(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 7)
        }
    }

    function addmod_zero(uint256 a, uint256 b, uint256 c) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := addmod(a, b, c)
            folded := addmod(5, 6, 0)
        }
    }

    function mulmod_no_wrap(uint256 a, uint256 b, uint256 c) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := mulmod(a, b, c)
            folded := mulmod(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 11)
        }
    }

    function mulmod_zero(uint256 a, uint256 b, uint256 c) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := mulmod(a, b, c)
            folded := mulmod(5, 6, 0)
        }
    }

    function slt_negative(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := slt(a, b)
            folded := slt(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 0)
        }
    }

    function lt_negative(uint256 a, uint256 b) external pure returns (uint256 live, uint256 folded) {
        assembly {
            live := lt(a, b)
            folded := lt(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, 0)
        }
    }
}
