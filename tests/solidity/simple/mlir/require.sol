//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "g(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x08c379a000000000000000000000000000000000000000000000000000000000",
//!                             "0x0000002000000000000000000000000000000000000000000000000000000000",
//!                             "0x00000006666f6f62617200000000000000000000000000000000000000000000",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "h(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x002ff06700000000000000000000000000000000000000000000000000000000",
//!                             "0x0000000200000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  error E(uint b);

  function f(bool a) public returns (bool) {
    require(a);
    return a;
  }
  function g(bool a) public returns (bool) {
    require(a, "foobar");
    return a;
  }
  function h(bool a) public returns (bool) {
    require(a, E(2));
    return a;
  }
}
