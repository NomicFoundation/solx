//! { "modes": [ "E" ], "cases": [ {
//!     "name": "pointerCallee",
//!     "inputs": [ { "method": "pointerCallee", "calldata": [] } ],
//!     "expected": [ "12" ]
//! }, {
//!     "name": "receiverOptions",
//!     "inputs": [ { "method": "receiverOptions", "calldata": [], "value": "10 wei" } ],
//!     "expected": [ "123" ]
//! }, {
//!     "name": "bareReceiverOptions",
//!     "inputs": [ { "method": "bareReceiverOptions", "calldata": [], "value": "10 wei" } ],
//!     "expected": [ "12" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    uint256 order;

    function t(uint256 n) internal returns (uint256) {
        order = order * 10 + n;
        return n;
    }

    function target(uint256 a) external payable returns (uint256) {
        return a;
    }

    function callee() internal returns (function(uint256) external payable returns (uint256)) {
        t(1);
        return this.target;
    }

    function receiver() internal returns (Test) {
        t(1);
        return this;
    }

    function bareReceiver() internal returns (address) {
        t(1);
        return address(this);
    }

    function pointerCallee() public returns (uint256) {
        order = 0;
        callee()(t(2));
        return order;
    }

    function receiverOptions() public payable returns (uint256) {
        order = 0;
        receiver().target{value: t(2)}(t(3));
        return order;
    }

    function bareReceiverOptions() public payable returns (uint256) {
        order = 0;
        bareReceiver().call{value: t(2)}("");
        return order;
    }
}
