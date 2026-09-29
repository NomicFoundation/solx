//! { "modes": [ "E" ], "cases": [ {
//!     "name": "operand",
//!     "inputs": [
//!         {
//!             "method": "operand",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "1"
//!     ]
//! }, {
//!     "name": "addresses",
//!     "inputs": [ { "method": "addresses", "calldata": [] } ],
//!     "expected": [ "5" ]
//! }, {
//!     "name": "arrays",
//!     "inputs": [ { "method": "arrays", "calldata": [] } ],
//!     "expected": [ "1", "0" ]
//! }, {
//!     "name": "byteArrays",
//!     "inputs": [ { "method": "byteArrays", "calldata": [] } ],
//!     "expected": [ "1", "0" ]
//! }, {
//!     "name": "reverting",
//!     "inputs": [ { "method": "reverting", "calldata": [] } ],
//!     "expected": { "return_data": [], "exception": true }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 evaluations;
    uint256[] data;
    bytes buffer;

    function recipient() internal returns (address payable) {
        evaluations += 1;
        return payable(address(this));
    }

    function operand() public returns (uint256) {
        evaluations = 0;
        recipient().transfer;
        return evaluations;
    }

    function addresses() public returns (uint256) {
        evaluations = 0;
        recipient().transfer;
        recipient().send;
        recipient().call;
        recipient().delegatecall;
        recipient().staticcall;
        return evaluations;
    }

    function array() internal returns (uint256[] storage) {
        evaluations += 1;
        return data;
    }

    function arrays() public returns (uint256, uint256) {
        evaluations = 0;
        array().pop;
        return (evaluations, data.length);
    }

    function byteArray() internal returns (bytes storage) {
        evaluations += 1;
        return buffer;
    }

    function byteArrays() public returns (uint256, uint256) {
        evaluations = 0;
        byteArray().pop;
        return (evaluations, buffer.length);
    }

    function failure() internal pure returns (address payable) {
        revert();
    }

    function reverting() public pure {
        failure().transfer;
    }
}
