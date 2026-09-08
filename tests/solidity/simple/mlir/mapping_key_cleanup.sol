//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "addressKey()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "uintKey()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "9"
//!                     ]
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    mapping(address => uint) addressMap;
    mapping(uint8 => uint) uintMap;

    function addressKey() public returns (uint) {
        address key;
        assembly {
            key := not(0)
        }
        addressMap[key] = 7;
        return addressMap[address(type(uint160).max)];
    }

    function uintKey() public returns (uint) {
        uint8 key;
        assembly {
            key := not(0)
        }
        uintMap[key] = 9;
        return uintMap[255];
    }
}
