//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "localIgnore()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": [
//!                         "7"
//!                     ]
//!                 },
//!                 {
//!                     "method": "callIgnore()",
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
    function pair() internal pure returns (uint256, uint256) {
        return (7, 9);
    }

    function localIgnore() public pure returns (uint256) {
        (uint256 x,) = pair();
        return x;
    }

    function callIgnore() public returns (bool) {
        (bool success,) = address(this).call(abi.encodeWithSignature("localIgnore()"));
        return success;
    }
}
