//! { "modes": [ "E" ], "cases": [ {
//!     "name": "constant",
//!     "inputs": [ { "method": "narrowed", "calldata": [] } ],
//!     "expected": [ "0x1ff" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  uint16 constant C = 0x1ff;

  function narrowed() external pure returns (uint256 r) {
    uint8 y = uint8(C);
    assembly { r := y }
  }
}
