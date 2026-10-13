//! { "modes": [ "E" ], "cases": [ {
//!     "name": "receiver",
//!     "inputs": [ { "method": "receiver", "calldata": [] } ],
//!     "expected": [ "12", "21" ]
//! }, {
//!     "name": "library_receiver_calldata",
//!     "inputs": [ { "method": "libraryReceiverCalldata", "calldata": [ "0x20", "1", "0xffffffff" ] } ],
//!     "expected": { "return_data": [], "exception": true }
//! }, {
//!     "name": "receiver_enum",
//!     "inputs": [ { "method": "receiverEnum", "calldata": [] } ],
//!     "expected": { "return_data": [ "0x4e487b7100000000000000000000000000000000000000000000000000000000", "0x0000002100000000000000000000000000000000000000000000000000000000" ], "exception": true }
//! }, {
//!     "name": "library_receiver_enum",
//!     "inputs": [ { "method": "libraryReceiverEnum", "calldata": [] } ],
//!     "expected": { "return_data": [ "0x4e487b7100000000000000000000000000000000000000000000000000000000", "0x0000002100000000000000000000000000000000000000000000000000000000" ], "exception": true }
//! } ], "libraries": { "attached_call.sol": { "Linked": "Linked" } } }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

enum Choice { First, Second }

library Pairs {
    function pair(uint256 x, uint256 y) internal pure returns (uint256) {
        return x * 10 + y;
    }

    function ordinal(Choice) internal pure returns (uint256) {
        return 7;
    }
}

library Linked {
    function size(string[] calldata self, uint256) public pure returns (uint256) {
        return self.length;
    }

    function linkedOrdinal(Choice, uint256) public pure returns (uint256) {
        return 7;
    }
}

contract Test {
    using Pairs for *;
    using Linked for *;

    uint256 order;

    function t(uint256 n) internal returns (uint256) {
        order = order * 10 + n;
        return n;
    }

    function fail() internal pure returns (uint256) {
        revert("fail");
    }

    function receiver() public returns (uint256, uint256) {
        order = 0;
        uint256 result = t(1).pair(t(2));
        return (result, order);
    }

    function libraryReceiverCalldata(string[] calldata values) public pure returns (uint256) {
        return values.size(fail());
    }

    function receiverEnum() public pure returns (uint256) {
        Choice dirty;
        assembly {
            dirty := 2
        }
        return dirty.ordinal();
    }

    function libraryReceiverEnum() public pure returns (uint256) {
        Choice dirty;
        assembly {
            dirty := 2
        }
        return dirty.linkedOrdinal(fail());
    }
}
