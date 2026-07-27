//! { "modes": [ "E" ], "cases": [ {
//!     "name": "arithmetic",
//!     "inputs": [ { "method": "arithmetic", "calldata": [] } ],
//!     "expected": [ "1", "34" ]
//! }, {
//!     "name": "bitwise",
//!     "inputs": [ { "method": "bitwise", "calldata": [] } ],
//!     "expected": [ "7", "34" ]
//! }, {
//!     "name": "comparison",
//!     "inputs": [ { "method": "comparison", "calldata": [] } ],
//!     "expected": [ "1", "34" ]
//! }, {
//!     "name": "exponentiation",
//!     "inputs": [ { "method": "exponentiation", "calldata": [] } ],
//!     "expected": [ "64", "34" ]
//! }, {
//!     "name": "shift",
//!     "inputs": [ { "method": "shift", "calldata": [] } ],
//!     "expected": [ "32", "34" ]
//! }, {
//!     "name": "addmod",
//!     "inputs": [ { "method": "addmodOrder", "calldata": [] } ],
//!     "expected": [ "2", "534" ]
//! }, {
//!     "name": "mulmod",
//!     "inputs": [ { "method": "mulmodOrder", "calldata": [] } ],
//!     "expected": [ "2", "534" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 sequence;

    function left() internal returns (uint256) {
        sequence = sequence * 10 + 4;
        return 4;
    }

    function right() internal returns (uint256) {
        sequence = sequence * 10 + 3;
        return 3;
    }

    function modulus() internal returns (uint256) {
        sequence = sequence * 10 + 5;
        return 5;
    }

    function arithmetic() public returns (uint256, uint256) {
        sequence = 0;
        uint256 result = left() - right();
        return (result, sequence);
    }

    function bitwise() public returns (uint256, uint256) {
        sequence = 0;
        uint256 result = left() ^ right();
        return (result, sequence);
    }

    function comparison() public returns (bool, uint256) {
        sequence = 0;
        bool result = left() > right();
        return (result, sequence);
    }

    function exponentiation() public returns (uint256, uint256) {
        sequence = 0;
        uint256 result = left() ** right();
        return (result, sequence);
    }

    function shift() public returns (uint256, uint256) {
        sequence = 0;
        uint256 result = left() << right();
        return (result, sequence);
    }

    function addmodOrder() public returns (uint256, uint256) {
        sequence = 0;
        uint256 result = addmod(left(), right(), modulus());
        return (result, sequence);
    }

    function mulmodOrder() public returns (uint256, uint256) {
        sequence = 0;
        uint256 result = mulmod(left(), right(), modulus());
        return (result, sequence);
    }
}
