//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "c()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                         "0x4300000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "a()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                         "0x4100000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "i()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                         "0x4900000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "names_match()",
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

abstract contract A {
  function fa() public virtual;
}

interface I {
  function fi() external;
}

contract C {}

contract Test {
  function c() public pure returns (string memory) {
    return type(C).name;
  }

  function a() public pure returns (string memory) {
    return type(A).name;
  }

  function i() public pure returns (string memory) {
    return type(I).name;
  }

  function names_match() public pure returns (bool) {
    return
        keccak256(bytes(type(C).name)) == keccak256(bytes("C")) &&
        keccak256(bytes(type(A).name)) == keccak256(bytes("A")) &&
        keccak256(bytes(type(I).name)) == keccak256(bytes("I"));
  }
}
