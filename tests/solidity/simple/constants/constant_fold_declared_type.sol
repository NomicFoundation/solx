//! {
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "digits",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x3031323334353637383961626364656600000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "hexDigit",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "0x3000000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "hexDigit",
//!                     "calldata": [
//!                         "15"
//!                     ],
//!                     "expected": [
//!                         "0x6600000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "pubDigit",
//!                     "calldata": [
//!                         "10"
//!                     ],
//!                     "expected": [
//!                         "0x4100000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "fileDigit",
//!                     "calldata": [
//!                         "0"
//!                     ],
//!                     "expected": [
//!                         "0x4500000000000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "text",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000004",
//!                         "0x736f6c7800000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 },
//!                 {
//!                     "method": "TEXT",
//!                     "calldata": [],
//!                     "expected": [
//!                         "0x0000000000000000000000000000000000000000000000000000000000000020",
//!                         "0x0000000000000000000000000000000000000000000000000000000000000004",
//!                         "0x736f6c7800000000000000000000000000000000000000000000000000000000"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }
// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

bytes16 constant FILE_DIGITS = "EDCBA98765432100";

contract Test {
    bytes16 private constant HEX_DIGITS = "0123456789abcdef";
    bytes16 public constant PUB_DIGITS = "0123456789ABCDEF";
    string public constant TEXT = "solx";

    function digits() external pure returns (bytes16) {
        return HEX_DIGITS;
    }

    function hexDigit(uint256 value) external pure returns (bytes1) {
        return HEX_DIGITS[value & 0xf];
    }

    function pubDigit(uint256 value) external pure returns (bytes1) {
        return PUB_DIGITS[value & 0xf];
    }

    function fileDigit(uint256 value) external pure returns (bytes1) {
        return FILE_DIGITS[value & 0xf];
    }

    function text() external pure returns (string memory) {
        return TEXT;
    }
}
