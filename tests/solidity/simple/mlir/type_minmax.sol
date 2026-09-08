//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "int8_min()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "-128"
//!                     ]
//!                 },
//!                 {
//!                     "method": "int8_max()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "127"
//!                     ]
//!                 },
//!                 {
//!                     "method": "uint8_min()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "uint8_max()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "255"
//!                     ]
//!                 },
//!                 {
//!                     "method": "int256_min()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "-57896044618658097711785492504343953926634992332820282019728792003956564819968"
//!                     ]
//!                 },
//!                 {
//!                     "method": "int256_max()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "57896044618658097711785492504343953926634992332820282019728792003956564819967"
//!                     ]
//!                 },
//!                 {
//!                     "method": "uint256_min()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "uint256_max()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "115792089237316195423570985008687907853269984665640564039457584007913129639935"
//!                     ]
//!                 },
//!                 {
//!                     "method": "enum_min()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "enum_max()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  enum E { A, B, Test }

  function int8_min() public pure returns (int8) {
    return type(int8).min;
  }

  function int8_max() public pure returns (int8) {
    return type(int8).max;
  }

  function uint8_min() public pure returns (uint8) {
    return type(uint8).min;
  }

  function uint8_max() public pure returns (uint8) {
    return type(uint8).max;
  }

  function int256_min() public pure returns (int256) {
    return type(int256).min;
  }

  function int256_max() public pure returns (int256) {
    return type(int256).max;
  }

  function uint256_min() public pure returns (uint256) {
    return type(uint256).min;
  }

  function uint256_max() public pure returns (uint256) {
    return type(uint256).max;
  }

  function enum_min() public pure returns (E) {
    return type(E).min;
  }

  function enum_max() public pure returns (E) {
    return type(E).max;
  }
}
