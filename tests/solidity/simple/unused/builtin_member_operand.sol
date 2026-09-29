//! { "cases": [ {
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
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 evaluations;

    function recipient() internal returns (address payable) {
        evaluations += 1;
        return payable(address(this));
    }

    function operand() public returns (uint256) {
        recipient().transfer;
        return evaluations;
    }
}
