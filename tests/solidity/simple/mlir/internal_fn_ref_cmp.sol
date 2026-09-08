//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "test_eq_same_fn()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "test_neq_diff_fn()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "test_eq_false_diff_fn()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "test_eq_stored()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "test_neq_stored()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Test {
    function(uint) internal pure returns (uint) stored;

    function f(uint x) internal pure returns (uint) { return x + 10; }
    function g(uint x) internal pure returns (uint) { return x + 20; }

    function test_eq_same_fn() public pure returns (bool) {
        return f == f;
    }

    function test_neq_diff_fn() public pure returns (bool) {
        return f != g;
    }

    function test_eq_false_diff_fn() public pure returns (bool) {
        return !(f == g);
    }

    function test_eq_stored() public returns (bool) {
        stored = f;
        return stored == f;
    }

    function test_neq_stored() public returns (bool) {
        stored = f;
        return stored != g;
    }
}
