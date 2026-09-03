//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "m()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ],
//!     "libraries": {
//!         "library.sol": {
//!             "L": "L"
//!         }
//!     }
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

library L {
  function f(uint a) internal returns (uint) { return a + 1; }
  function g(uint a) external returns (uint) { return f(a); }
}

contract Test {
  function m() external returns (uint) { return L.g(1); }
}
