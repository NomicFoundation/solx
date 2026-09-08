//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "m(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0",
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "11"
//!                     ]
//!                 },
//!                 {
//!                     "method": "m(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1",
//!                         "2"
//!                     ],
//!                     "expected": [
//!                         "22"
//!                     ]
//!                 },
//!                 {
//!                     "method": "m(uint256,uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "2",
//!                         "3"
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
//!                     "method": "n(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "10"
//!                     ]
//!                 },
//!                 {
//!                     "method": "n(bool)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
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
  function f(uint a) internal returns (uint) { return a + 10; }
  function g(uint a) internal returns (uint) { return a + 20; }

  // Select and call an internal function pointer.
  function m(uint a, uint b) public returns (uint) {
    function(uint) internal returns (uint) p;
    if (a == 0)
      p = f;
    else if (a == 1)
      p = g;
    return p(b);
  }

  // Internal function pointer as state variable.
  function (uint) internal returns (uint) s0;
  function (uint) internal returns (uint) s1 = f;
  function n(bool a) public returns (uint) {
    if (a)
      return s1(0);
    return s0(0);
  }
}
