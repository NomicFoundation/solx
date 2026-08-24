//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": []
//!                 },
//!                 {
//!                     "method": "g()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 },
//!                 {
//!                     "method": "h()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42",
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "i0(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "i1(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "3"
//!                     ]
//!                 },
//!                 {
//!                     "method": "b(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "i8(uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "255"
//!                     ],
//!                     "expected": [
//!                         "255"
//!                     ]
//!                 },
//!                 {
//!                     "method": "i8(uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "256"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "s()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "42"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  uint public s;
  uint[2][2] public si2;

  string public str;
  constructor () {
  }
  function f() public { s = 42; }
  function g() public returns (uint) { return s; }
  function h() public returns (uint, uint) { return (s, 1); }
  function i0(uint a) public returns (uint) { return a; }
  function i1(uint a, uint b) public returns (uint) { return a + b; }
  function b(bool a) public returns (bool) { return a; }
  function i8(uint8 a) public returns (uint8) { return a; }
}
