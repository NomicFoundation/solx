//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "Dm()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "Cm()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "Cn()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "11"
//!                     ]
//!                 },
//!                 {
//!                     "method": "Dn()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract C {
  uint public Cm;
  uint public Cn;
  constructor () {
    Cm = 10;
    Cn = 11;
  }

  function f() public returns (uint) { return Cm; }
}

contract Test is C {
  uint public Dm = f();
  uint public Dn = Cn;
}
