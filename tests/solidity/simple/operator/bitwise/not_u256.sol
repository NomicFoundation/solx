//! { "cases": [ {
//!     "name": "ordinary",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "42"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "115792089237316195423570985008687907853269984665640564039457584007913129639893"
//!     ]
//! }, {
//!     "name": "min_to_max",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "0"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "115792089237316195423570985008687907853269984665640564039457584007913129639935"
//!     ]
//! }, {
//!     "name": "max_to_min",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                 "115792089237316195423570985008687907853269984665640564039457584007913129639935"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(uint256 a) public pure returns(uint256) {
        return ~a;
    }
}
