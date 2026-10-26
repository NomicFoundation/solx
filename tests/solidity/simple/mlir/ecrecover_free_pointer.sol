//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "f()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                             "0x00000000000000000000000043c41e805bdc6219f7141a47103a7408eb50e841"
//!                         ]
//!                     }
//!                 },
//!                 {
//!                     "method": "g()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [
//!                             "0x0000000000000000000000000000000000000000000000000000000000000001",
//!                             "0x0000000000000000000000000000000000000000000000000000000000000000"
//!                         ]
//!                     }
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function f() public view returns (bool x, address a) {
        uint256 p;
        uint256 q;
        assembly {
            q := mload(0x40)
        }
        a = ecrecover(bytes32(uint256(1)), 27, bytes32(uint256(2)), bytes32(uint256(3)));
        assembly {
            p := mload(0x40)
        }
        x = p - q <= 32;
    }

    function g() public view returns (bool x, address a) {
        uint256 p;
        uint256 q;
        assembly {
            q := mload(0x40)
        }
        a = ecrecover(bytes32(0), 0, bytes32(0), bytes32(0));
        assembly {
            p := mload(0x40)
        }
        x = p - q <= 32;
    }
}
