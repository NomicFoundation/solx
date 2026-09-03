//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "int24_min()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "-8388608"
//!                     ]
//!                 },
//!                 {
//!                     "method": "int24_max()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "8388607"
//!                     ]
//!                 },
//!                 {
//!                     "method": "int72_min()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "-2361183241434822606848"
//!                     ]
//!                 },
//!                 {
//!                     "method": "int72_max()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2361183241434822606847"
//!                     ]
//!                 },
//!                 {
//!                     "method": "int136_min()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "-43556142965880123323311949751266331066368"
//!                     ]
//!                 },
//!                 {
//!                     "method": "int136_max()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "43556142965880123323311949751266331066367"
//!                     ]
//!                 },
//!                 {
//!                     "method": "rational_shr()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "66"
//!                     ]
//!                 },
//!                 {
//!                     "method": "rational_sub()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2361183241434822606847"
//!                     ]
//!                 },
//!                 {
//!                     "method": "rational_lt()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "rational_eq()",
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

pragma solidity >=0.8.0;

contract Test {
  function int24_min() public pure returns (int24) { return type(int24).min; }

  function int24_max() public pure returns (int24) { return type(int24).max; }

  function int72_min() public pure returns (int72) { return type(int72).min; }

  function int72_max() public pure returns (int72) { return type(int72).max; }

  function int136_min() public pure returns (int136) { return type(int136).min; }

  function int136_max() public pure returns (int136) { return type(int136).max; }

  function rational_shr() public pure returns (uint8) { return 0x4200 >> 8; }

  function rational_sub() public pure returns (int72) { return 2**71 - 1; }

  function rational_lt() public pure returns (bool) { return 1 < 2; }

  function rational_eq() public pure returns (bool) { return 3 == 3; }
}
