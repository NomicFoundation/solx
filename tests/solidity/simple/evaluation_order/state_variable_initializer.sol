//! { "modes": [ "E" ], "cases": [ {
//!     "name": "callvalue",
//!     "inputs": [ { "method": "#deployer", "calldata": [], "value": "1 wei" } ],
//!     "expected": { "return_data": [ "0x08c379a000000000000000000000000000000000000000000000000000000000", "0x0000002000000000000000000000000000000000000000000000000000000000", "0x0000000576616c75650000000000000000000000000000000000000000000000", "0x0000000000000000000000000000000000000000000000000000000000000000" ], "exception": true }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 initialized = initialize();

    function initialize() internal returns (uint256) {
        require(msg.value == 0, "value");
        return 1;
    }
}
