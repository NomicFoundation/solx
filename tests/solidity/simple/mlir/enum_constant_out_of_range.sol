//! { "cases": [ {
//!     "name": "in_range",
//!     "inputs": [ { "method": "inRange", "calldata": [] } ],
//!     "expected": [ "2" ]
//! }, {
//!     "name": "out_of_range",
//!     "inputs": [ { "method": "outOfRange", "calldata": [] } ],
//!     "expected": {
//!         "return_data": [
//!             "0x4e487b7100000000000000000000000000000000000000000000000000000000",
//!             "0x0000002100000000000000000000000000000000000000000000000000000000"
//!         ],
//!         "exception": true
//!     }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

// A conversion of a named constant to an enum keeps the range check that a
// literal, which the frontend rejects out of range, does not need.
contract Test {
    enum E { A, B, C }
    uint8 constant IN = 2;
    uint8 constant OUT = 5;

    function inRange() external pure returns (E) {
        return E(IN);
    }

    function outOfRange() external pure returns (E) {
        return E(OUT);
    }
}
