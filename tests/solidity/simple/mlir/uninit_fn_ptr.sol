//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "a(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000005100000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "b()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000005100000000000000000000000000000000000000000000000000000000"
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
  // Never assigned: their types have no candidate functions, so both calls
  // dispatch over an empty set.
  function (uint) internal returns (uint) f1;
  function () internal f2;

  function a(uint x) public returns (uint) { return f1(x); }
  function b() public { f2(); }
}
