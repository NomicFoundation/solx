//! { "cases": [ {
//!     "name": "by_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                "1",
//!                "2",
//!                "0"
//!             ]
//!         }
//!     ],
//!     "expected": [ {
//!         "compiler_version": "<=0.4.20",
//!         "return_data": [
//!             "0"
//!         ],
//!         "exception": false
//!     }, {
//!         "compiler_version": ">=0.4.21,<0.8",
//!         "return_data": [],
//!         "exception": true
//!     }, {
//!         "compiler_version": ">=0.8",
//!         "return_data": [
//!             "0x4E487B7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000001200000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     } ]
//! }, {
//!     "name": "ordinary_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                "42",
//!                "42",
//!                "1764"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "ordinary_non_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                "41",
//!                "2",
//!                "40"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "2"
//!     ]
//! }, {
//!     "name": "overflow_minimal_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
//!                "2",
//!                "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0"
//!     ]
//! }, {
//!     "name": "overflow_minimal_non_zero",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE",
//!                "2",
//!                "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFD"
//!     ]
//! }, {
//!     "name": "overflow",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!                "0x1F1E1D1C1B1A191817161514131211100F0E0D0C0B0A09080706050403020100",
//!                "0x3F3E3D3C3B3A393837363534333231302F2E2D2C2B2A29282726252423222120",
//!                "0x000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "0x000000000000000000584D2AF1A1416B1F5F2B856DE5EE852CE4AB806250494C"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract Test {
    function main(uint x, uint y, uint m) public pure returns(uint) {
        return mulmod(x, y, m);
    }
}
