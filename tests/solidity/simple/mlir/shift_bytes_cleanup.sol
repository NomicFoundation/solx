//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "l(uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "64"
//!                     ],
//!                     "expected": [
//!                         "0x3930313233343536373839300000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "r(uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "64"
//!                     ],
//!                     "expected": [
//!                         "0x313233343536373839303132000000000000000000000000"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  function l(uint8 y) public returns (bytes20) {
    bytes20 x;
    assembly { x := "12345678901234567890abcde" }
    return x << y;
  }
  function r(uint8 y) public returns (bytes20) {
    bytes20 x;
    assembly { x := "12345678901234567890abcde" }
    return x >> y;
  }
}
