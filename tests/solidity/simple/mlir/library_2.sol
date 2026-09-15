//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "l()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ],
//!     "libraries": {
//!         "library_2.sol": {
//!             "L": "L"
//!         }
//!     }
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

library L {
  function f(uint a) external view returns (uint) { return a; }
}

contract Test {
  function l() external returns (uint) { return L.f(1); }
}
