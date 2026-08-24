//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "fromMemory16(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000010",
//!                         "0x6162636465666768616263646566676800000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0x6162636465666768616263646566676800000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fromMemoryCleanup(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000010",
//!                         "0x6162636465666768616263646566676800000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0x6162636465666768616263646566000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fromMemory32(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fromCalldata16(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x000000000000000000000000000000000000000000000000000000000000000f",
//!                         "0x6162636465666768616263646566676800000000000000000000000000000000"
//!                     ],
//!                     "expected": [
//!                         "0x6162636465666768616263646566670000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fromCalldata32(bytes)",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [
//!                         "0x20",
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fromStorageEmpty()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fromStorageShort()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x6162636465666768616263646566676861626364656667686162636465666700"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fromStorageLong()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x6162636465666768616263646566676861626364656667686162636465666768"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
  bytes emptyStorage = "";
  bytes shortStorage = "abcdefghabcdefghabcdefghabcdefg";
  bytes longStorage = "abcdefghabcdefghabcdefghabcdefghX";

  function fromMemory16(bytes memory m) public pure returns (bytes16) {
    return bytes16(m);
  }

  function fromMemoryCleanup(bytes memory m) public pure returns (bytes16) {
    assembly {
      mstore(m, 14)
    }
    return bytes16(m);
  }

  function fromMemory32(bytes memory m) public pure returns (bytes32) {
    return bytes32(m);
  }

  function fromCalldata16(bytes calldata c) external pure returns (bytes16) {
    return bytes16(c);
  }

  function fromCalldata32(bytes calldata c) external pure returns (bytes32) {
    return bytes32(c);
  }

  function fromStorageEmpty() external view returns (bytes32) {
    return bytes32(emptyStorage);
  }

  function fromStorageShort() external view returns (bytes32) {
    return bytes32(shortStorage);
  }

  function fromStorageLong() external view returns (bytes32) {
    return bytes32(longStorage);
  }
}
