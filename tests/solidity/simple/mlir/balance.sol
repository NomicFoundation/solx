//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "selfbalance()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "value": "23 wei",
//!                     "expected": [
//!                         "23"
//!                     ]
//!                 },
//!                 {
//!                     "method": "balance()",
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

contract D {
  constructor() payable {}
}

contract Test {
  function selfbalance() public payable returns (uint) {
    return address(this).balance;
  }

  function balance() public payable returns (uint) {
    D d = new D{value: 7 wei}();
    return address(d).balance;
  }
}
