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
//!                             "0x0000000000000000000000000000000000000000000000000000000000000007"
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
    function g() internal pure returns (uint256[1] memory) {
        uint256 x;
        assembly {
            x := sub(mload(0x40), 0x20)
            mstore(x, 7)
        }
    }

    function f() public pure returns (uint256) {
        return g()[0];
    }
}
