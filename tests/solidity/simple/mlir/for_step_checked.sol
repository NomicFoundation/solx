//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "counter()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "5"
//!                     ]
//!                 },
//!                 {
//!                     "method": "inclusive(uint8)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "255"
//!                     ],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "compound()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "decrement()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "widened()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "bodyWrite()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!                             "0x0000001100000000000000000000000000000000000000000000000000000000"
//!                         ],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "uncheckedStep()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  function counter() public pure returns (uint8) {
    uint8 last = 0;
    for (uint8 i = 0; i < 5; ++i) {
      last = i + 1;
    }
    return last;
  }

  function inclusive(uint8 limit) public pure returns (uint8) {
    uint8 last = 0;
    for (uint8 i = 0; i <= limit; ++i) {
      last = i;
    }
    return last;
  }

  function compound() public pure returns (uint8) {
    uint8 last = 0;
    for (uint8 i = 250; i < 255; i += 10) {
      last = i;
    }
    return last;
  }

  function decrement() public pure returns (uint8) {
    uint8 last = 0;
    for (uint8 i = 0; i < 5; --i) {
      last = i;
    }
    return last;
  }

  function widened() public pure returns (uint8) {
    uint256 limit = 300;
    uint8 last = 0;
    for (uint8 i = 0; i < limit; ++i) {
      last = i;
    }
    return last;
  }

  function bodyWrite() public pure returns (uint8) {
    uint8 last = 0;
    for (uint8 i = 0; i < 5; ++i) {
      last = i;
      i = 255;
    }
    return last;
  }

  // An unchecked block still wraps the step.
  function uncheckedStep() public pure returns (uint8) {
    unchecked {
      uint8 last = 0;
      for (uint8 i = 254; i != 3; ++i) {
        last = i;
      }
      return last;
    }
  }
}
