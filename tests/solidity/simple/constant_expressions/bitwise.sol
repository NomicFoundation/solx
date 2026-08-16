//! { "cases": [ {
//!     "name": "test",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x10000000000000000000000001"
//!     ]
//! }, {
//!     "name": "shifts",
//!     "inputs": [
//!         {
//!             "method": "shifts",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x722000"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function test() public pure returns (uint128) {
        return ~0&1<<100|257^256;
    }

    function shifts() public pure returns (uint256) {
        return 0x722DDB>>12<<1000>>988;
    }
}
