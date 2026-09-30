//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "length2(bytes2)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x4e4c000000000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "2"
//!                     ]
//!                 },
//!                 {
//!                     "method": "length32(bytes32)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "32"
//!                     ]
//!                 },
//!                 {
//!                     "method": "lengthOfExpression(bytes4,bytes4)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x1234567800000000000000000000000000000000000000000000000000000000",
//!                         "0x8765432100000000000000000000000000000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "4"
//!                     ]
//!                 },
//!                 {
//!                     "method": "lengthEvaluatesOperand()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "9"
//!                     ]
//!                 },
//!                 {
//!                     "method": "hasCountry()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  struct Details {
    bytes2 country;
  }

  Details details;
  uint256 counter;

  function length2(bytes2 b) public pure returns (uint256) {
    return b.length;
  }

  function length32(bytes32 b) public pure returns (uint256) {
    return b.length;
  }

  function lengthOfExpression(bytes4 a, bytes4 b) public pure returns (uint256) {
    return (a ^ b).length;
  }

  function next() internal returns (bytes8) {
    counter += 1;
    return bytes8(0);
  }

  function lengthEvaluatesOperand() public returns (uint256) {
    uint256 length = next().length;
    return length + counter;
  }

  function hasCountry() public view returns (bool) {
    return details.country.length != 0;
  }
}
