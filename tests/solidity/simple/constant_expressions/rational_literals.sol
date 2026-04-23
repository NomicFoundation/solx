//! { "cases": [ {
//!     "name": "half_minute",
//!     "inputs": [
//!         {
//!             "method": "half_minute",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "30"
//!     ]
//! }, {
//!     "name": "half_gwei",
//!     "inputs": [
//!         {
//!             "method": "half_gwei",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "500000000"
//!     ]
//! }, {
//!     "name": "quarter_ether",
//!     "inputs": [
//!         {
//!             "method": "quarter_ether",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "250000000000000000"
//!     ]
//! }, {
//!     "name": "half_ether",
//!     "inputs": [
//!         {
//!             "method": "half_ether",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "500000000000000000"
//!     ]
//! }, {
//!     "name": "three_halves_ether",
//!     "inputs": [
//!         {
//!             "method": "three_halves_ether",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1500000000000000000"
//!     ]
//! }, {
//!     "name": "scientific_thousand",
//!     "inputs": [
//!         {
//!             "method": "scientific_thousand",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1000"
//!     ]
//! }, {
//!     "name": "scientific_fractional",
//!     "inputs": [
//!         {
//!             "method": "scientific_fractional",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1500"
//!     ]
//! }, {
//!     "name": "scientific_large",
//!     "inputs": [
//!         {
//!             "method": "scientific_large",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "1000000000000000000"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    function half_minute() public pure returns (uint256) {
        return 0.5 minutes;
    }

    function half_gwei() public pure returns (uint256) {
        return 0.5 gwei;
    }

    function quarter_ether() public pure returns (uint256) {
        return 0.25 ether;
    }

    function half_ether() public pure returns (uint256) {
        return 0.5 ether;
    }

    function three_halves_ether() public pure returns (uint256) {
        return 1.5 ether;
    }

    function scientific_thousand() public pure returns (uint256) {
        return 1e3;
    }

    function scientific_fractional() public pure returns (uint256) {
        return 1.5e3;
    }

    function scientific_large() public pure returns (uint256) {
        return 1e18;
    }
}
