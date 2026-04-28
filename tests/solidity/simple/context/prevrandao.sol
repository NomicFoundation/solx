//! { "cases": [ {
//!     "name": "main",
//!     "inputs": [
//!         {
//!             "method": "main",
//!             "calldata": [
//!             ]
//!         }
//!     ],
//!     "expected": [
//!         "$PREVRANDAO"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.18;

contract Test {
    function main() public view returns(uint) {
        uint _prevrandao = block.prevrandao;
        return _prevrandao;
    }
}
