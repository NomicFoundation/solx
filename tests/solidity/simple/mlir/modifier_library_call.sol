//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "5"
//!                     ],
//!                     "expected": [
//!                         "6"
//!                     ]
//!                 },
//!                 {
//!                     "method": "f(uint256)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "10"
//!                     ],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

library L {
  function check(uint x) internal pure returns (bool) {
    return x < 10;
  }
}

contract Test {
  modifier m(uint x) {
    require(L.check(x));
    _;
  }

  function f(uint x) public m(x) returns (uint) {
    return x + 1;
  }
}
