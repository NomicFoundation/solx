//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "directGetter()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "11"
//!                     ]
//!                 },
//!                 {
//!                     "method": "getterPointer()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "11"
//!                     ]
//!                 },
//!                 {
//!                     "method": "getterStruct()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "11"
//!                     ]
//!                 },
//!                 {
//!                     "method": "functionStruct()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  uint public value = 11;

  struct S {
    function() external view returns (uint) fn;
  }

  function g() public view returns (uint) {
    return 7;
  }

  function directGetter() public view returns (uint) {
    return this.value();
  }

  function getterPointer() public view returns (uint) {
    function() external view returns (uint) fp = this.value;
    return fp();
  }

  function getterStruct() public view returns (uint) {
    S memory s;
    s.fn = this.value;
    return s.fn();
  }

  function functionStruct() public view returns (uint) {
    S memory s;
    s.fn = this.g;
    return s.fn();
  }
}
