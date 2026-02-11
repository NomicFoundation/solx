//! { "cases": [ {
//!     "name": "create2_deterministic_address",
//!     "inputs": [
//!         {
//!             "method": "test",
//!             "calldata": []
//!         }
//!     ],
//!     "expected": [
//!         "0x01"
//!     ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Child {
    uint256 public value;

    constructor() {
        value = 42;
    }
}

contract Test {
    function test() public returns (bool) {
        bytes32 salt = bytes32(uint256(1));

        // Compute expected address
        address predicted = address(uint160(uint256(keccak256(abi.encodePacked(
            bytes1(0xff),
            address(this),
            salt,
            keccak256(type(Child).creationCode)
        )))));

        // Deploy with CREATE2
        Child child = new Child{salt: salt}();

        return address(child) == predicted;
    }
}
