//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "runtime_code_length()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "runtime_code_compare()",
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

contract A {
  function f() public {}
}

contract Test {
  function runtime_code_length() public pure returns (bool) {
    return type(A).runtimeCode.length > 0;
  }

  function runtime_code_compare() public pure returns (bool) {
    bytes memory code = type(A).runtimeCode;
    return keccak256(code) == keccak256(type(A).runtimeCode);
  }
}
