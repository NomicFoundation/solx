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
//! }, {
//!     "name": "callOptions",
//!     "inputs": [ { "method": "callOptions", "calldata": [] } ],
//!     "expected": [ "123" ]
//! }, {
//!     "name": "externalOptions",
//!     "inputs": [ { "method": "externalOptions", "calldata": [] } ],
//!     "expected": [ "12" ]
//! }, {
//!     "name": "creationOptions",
//!     "inputs": [ { "method": "creationOptions", "calldata": [] } ],
//!     "expected": [ "12" ]
//! }, {
//!     "name": "revertingOption",
//!     "inputs": [ { "method": "revertingOption", "calldata": [] } ],
//!     "expected": { "return_data": [], "exception": true }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Child {
    constructor() payable {
        revert();
    }
}

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

    function option(uint256 digit) internal returns (uint256) {
        evaluations = evaluations * 10 + digit;
        return 0;
    }

    function callOptions() public returns (uint256) {
        evaluations = 0;
        recipient().call{value: option(2), gas: option(3)};
        return evaluations;
    }

    function externalOptions() public returns (uint256) {
        evaluations = 0;
        Test(recipient()).operand{gas: option(2)};
        return evaluations;
    }

    function creationOptions() public returns (uint256) {
        evaluations = 0;
        new Child{salt: bytes32(option(1)), value: option(2)};
        return evaluations;
    }

    function failingOption() internal pure returns (uint256) {
        revert();
    }

    function revertingOption() public {
        recipient().call{value: failingOption()};
    }
}
