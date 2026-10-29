//! { "cases": [ {
//!     "name": "literal",
//!     "inputs": [ { "method": "literal", "calldata": [] } ],
//!     "expected": [ "0x183a6125c38840424c4a85fa12bab2ab606c4b6d0e7cc73c0c06ba5300eab500" ]
//! }, {
//!     "name": "memory",
//!     "inputs": [ {
//!         "method": "fromMemory",
//!         "calldata": [ "0x20", "12", "0x6578616d706c652e6d61696e0000000000000000000000000000000000000000" ]
//!     } ],
//!     "expected": [ "0x183a6125c38840424c4a85fa12bab2ab606c4b6d0e7cc73c0c06ba5300eab500" ]
//! }, {
//!     "name": "calldata",
//!     "inputs": [ {
//!         "method": "fromCalldata",
//!         "calldata": [ "0x20", "12", "0x6578616d706c652e6d61696e0000000000000000000000000000000000000000" ]
//!     } ],
//!     "expected": [ "0x183a6125c38840424c4a85fa12bab2ab606c4b6d0e7cc73c0c06ba5300eab500" ]
//! }, {
//!     "name": "storage",
//!     "inputs": [ { "method": "fromStorage", "calldata": [] } ],
//!     "expected": [ "0x183a6125c38840424c4a85fa12bab2ab606c4b6d0e7cc73c0c06ba5300eab500" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.35;

contract Test {
    string name = "example.main";

    function literal() public pure returns (uint256) {
        return erc7201("example.main");
    }

    function fromMemory(string memory namespace) public pure returns (uint256) {
        return erc7201(namespace);
    }

    function fromCalldata(string calldata namespace) external pure returns (uint256) {
        return erc7201(namespace);
    }

    function fromStorage() public view returns (uint256) {
        return erc7201(name);
    }
}
