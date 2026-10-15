//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "name()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                         "0x4300000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "interface_id()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0xc61b20e200000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "creation_code()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "1"
//!                     ]
//!                 },
//!                 {
//!                     "method": "runtime_code()",
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

interface I {
  function fi() external;
}

contract C {}

contract Test {
  function name() public pure returns (string memory) {
    return (type(C)).name;
  }

  function interface_id() public pure returns (bytes4) {
    return (type(I)).interfaceId;
  }

  function creation_code() public pure returns (bool) {
    bytes memory code = (type(C)).creationCode;
    return code.length > 0 && keccak256(code) == keccak256(type(C).creationCode);
  }

  function runtime_code() public pure returns (bool) {
    bytes memory code = ((type(C))).runtimeCode;
    return code.length > 0 && keccak256(code) == keccak256(type(C).runtimeCode);
  }
}
