//! {
//!     "modes": [
//!         "E"
//!     ],
//!     "cases": [
//!         {
//!             "name": "default",
//!             "inputs": [
//!                 {
//!                     "method": "create()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 },
//!                 {
//!                     "method": "create2()",
//!                     "caller": "0x1212121212121212121212121212120000000012",
//!                     "calldata": [],
//!                     "expected": {
//!                         "return_data": [],
//!                         "exception": true
//!                     }
//!                 }
//!             ]
//!         }
//!     ]
//! }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Failer {
    constructor() {
        revert();
    }
}

contract Test {
    function create() public {
        new Failer();
    }

    function create2() public {
        new Failer{salt: bytes32(uint256(0x10))}();
    }
}
