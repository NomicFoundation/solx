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
//!                     "method": "test_neq_diff_sel()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "test_eq_false_diff_sel()",
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
//!                 },
//!                 {
//!                     "method": "test_neq_diff_addr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "test_eq_ext_contract()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "test_neq_ext_contract()",
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

contract Helper {
    function a() external pure returns (uint) { return 1; }
    function b() external pure returns (uint) { return 2; }
}

contract Test {
    function() external pure returns (uint) stored;

    function f() external pure returns (uint) { return 10; }
    function g() external pure returns (uint) { return 20; }

    function test_eq_same_fn() public returns (bool) {
        return this.f == this.f;
    }

    function test_neq_diff_sel() public returns (bool) {
        return this.f != this.g;
    }

    function test_eq_false_diff_sel() public returns (bool) {
        return !(this.f == this.g);
    }

    function test_eq_stored() public returns (bool) {
        stored = this.f;
        return stored == this.f;
    }

    function test_neq_stored() public returns (bool) {
        stored = this.f;
        return stored != this.g;
    }

    function test_neq_diff_addr() public returns (bool) {
        Helper h1 = new Helper();
        Helper h2 = new Helper();
        return h1.a != h2.a;
    }

    function test_eq_ext_contract() public returns (bool) {
        Helper h = new Helper();
        return h.a == h.a;
    }

    function test_neq_ext_contract() public returns (bool) {
        Helper h = new Helper();
        return h.a != h.b;
    }
}
