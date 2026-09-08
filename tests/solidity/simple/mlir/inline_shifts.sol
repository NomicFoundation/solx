//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "shl_asm()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "8"
//!                     ]
//!                 },
//!                 {
//!                     "method": "shr_asm()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "sar_asm()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "-2"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  function shl_asm() public pure returns (uint) {
    uint r;
    assembly {
      r := shl(1, 4)
    }
    return r;
  }

  function shr_asm() public pure returns (uint) {
    uint r;
    assembly {
      r := shr(1, 4)
    }
    return r;
  }

  function sar_asm() public pure returns (int) {
    int r;
    assembly {
      r := sar(1, sub(0, 4))
    }
    return r;
  }
}
