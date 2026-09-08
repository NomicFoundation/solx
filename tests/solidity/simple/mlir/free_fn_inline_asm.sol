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

function double(uint x) pure returns (uint r) {
  assembly {
    function g(a) -> b {
      b := mul(a, 2)
    }
    r := g(x)
  }
}

contract Test {
  function f() public pure returns (uint) {
    return double(21);
  }
}
