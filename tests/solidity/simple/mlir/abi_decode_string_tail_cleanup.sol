//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "tail_after_decode_memory()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  function tail_after_decode_memory() public returns (uint256) {
    bytes memory enc = abi.encode("a");
    uint256 tail;

    assembly {
      // Force decode allocation to reuse this dirty chunk.
      let p := mload(0x40)
      mstore(add(p, 0x20), not(0))
      mstore(add(p, 0x40), not(0))
      mstore(0x40, p)
    }

    string memory s = abi.decode(enc, (string));

    assembly {
      tail := mload(add(add(s, 0x20), mload(s)))
    }
    return tail;
  }
}
