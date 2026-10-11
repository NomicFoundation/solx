//! { "cases": [ {
//!     "name": "remainder",
//!     "inputs": [
//!         {
//!             "method": "remainder",
//!             "calldata": [
//!                 "5", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001200000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! }, {
//!     "name": "unchecked_division",
//!     "inputs": [
//!         {
//!             "method": "uncheckedDivision",
//!             "calldata": [
//!                 "5", "0"
//!             ]
//!         }
//!     ],
//!     "expected": {
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001200000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function remainder(uint256 a, uint256 b) public pure returns(uint256) {
        a % b;
        return 1;
    }

    function uncheckedDivision(uint256 a, uint256 b) public pure returns(uint256) {
        unchecked {
            a / b;
        }
        return 1;
    }
}
