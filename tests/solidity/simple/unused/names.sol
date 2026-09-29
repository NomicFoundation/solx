//! { "modes": [ "E" ], "cases": [ {
//!     "name": "builtins",
//!     "inputs": [ { "method": "builtins", "calldata": [] } ],
//!     "expected": [ "1" ]
//! }, {
//!     "name": "types",
//!     "inputs": [ { "method": "types", "calldata": [] } ],
//!     "expected": [ "2" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.24;

type Amount is uint256;

contract Test {
    struct Pair {
        uint256 first;
    }

    enum Choice { First, Second }

    function builtins() public pure returns (uint256) {
        abi;
        block;
        msg;
        tx;
        addmod;
        mulmod;
        assert;
        blobhash;
        blockhash;
        ecrecover;
        gasleft;
        keccak256;
        ripemd160;
        selfdestruct;
        sha256;
        abi.encode;
        abi.decode;
        abi.encodePacked;
        abi.encodeWithSelector;
        abi.encodeWithSignature;
        Amount.wrap;
        Amount.unwrap;
        bytes.concat;
        string.concat;
        return 1;
    }

    function types() public pure returns (uint256) {
        uint256;
        address;
        bytes;
        string;
        Pair;
        Pair[7][];
        Amount;
        Choice;
        type(uint256);
        type(Choice);
        type(Test);
        (type(uint256));
        return 2;
    }
}
