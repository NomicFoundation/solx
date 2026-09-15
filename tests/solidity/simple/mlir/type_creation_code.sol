//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "creation_code_length()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "creation_code_compare()",
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
  function creation_code_length() public pure returns (bool) {
    return type(A).creationCode.length > 0;
  }

  function creation_code_compare() public pure returns (bool) {
    bytes memory code = type(A).creationCode;
    return keccak256(code) == keccak256(type(A).creationCode);
  }
}
