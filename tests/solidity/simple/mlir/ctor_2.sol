//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "#deployer",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "1"
//!                     ],
//!                     "expected": [
//!                         "Test.address"
//!                     ]
//!                 },
//!                 {
//!                     "method": "m()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "421"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract E {
  uint public m;
  constructor(uint a) { m += a * 100; }
}

contract D is E {
  constructor(uint a, uint b) E(a + 2) { m += a * 10; }
}

contract Test is D {
  constructor(uint a) D(a + 1, a + 9) { m += a; }
}
