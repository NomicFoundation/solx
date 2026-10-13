//! { "modes": [ "E" ], "cases": [ {
//!     "name": "indexed",
//!     "inputs": [ { "method": "indexedArguments", "calldata": [] } ],
//!     "expected": { "return_data": [ "21" ], "events": [ { "topics": [ "0xa35c51d50c6df200ed4206af91013d98532403835d8ccef2d52c1d76a402dae8", "1", "2" ], "values": [] } ] }
//! }, {
//!     "name": "indexed_reference",
//!     "inputs": [ { "method": "indexedReference", "calldata": [] } ],
//!     "expected": { "return_data": [], "events": [ { "topics": [ "0xa0b215eeaeda1f2b5a046e36a6817e1c3b44b95abfc4236dc18317e6d0c54324", "0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6" ], "values": [ "7" ] } ] }
//! }, {
//!     "name": "copy",
//!     "inputs": [ { "method": "copy", "calldata": [] } ],
//!     "expected": { "return_data": [], "events": [ { "topics": [ "0xa1fba123b3e5154c21b3b49d694f05d20a0856f55ce0a5fa2c12b2bb2903ac16" ], "values": [ "64", "1", "1", "77" ] } ] }
//! }, {
//!     "name": "indexed_reference_order",
//!     "inputs": [ { "method": "indexedReferenceOrder", "calldata": [] } ],
//!     "expected": { "return_data": [], "events": [ { "topics": [ "0xe4d1b8460b95d210e181dac706a42a883cb965bc0df74ac715fc22a156e41687", "7", "0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6" ], "values": [] } ] }
//! }, {
//!     "name": "indexed_enum",
//!     "inputs": [ { "method": "indexedEnum", "calldata": [] } ],
//!     "expected": { "return_data": [ "0x4e487b7100000000000000000000000000000000000000000000000000000000", "0x0000002100000000000000000000000000000000000000000000000000000000" ], "exception": true }
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Test {
    enum Choice { First, Second }

    uint256 order;
    uint256[] array;

    event Indexed(uint256 indexed first, uint256 indexed second);
    event Hashed(uint256[] indexed values, uint256 value);
    event Copied(uint256[] values, uint256 length);
    event Pair(uint256 indexed value, uint256[] indexed values);
    event Chosen(uint256 indexed value, Choice indexed choice);

    function t(uint256 n) internal returns (uint256) {
        order = order * 10 + n;
        return n;
    }

    function mutate(uint256[] memory values) internal pure returns (uint256) {
        values[0] = 99;
        return 7;
    }

    function grow() internal returns (uint256) {
        array.push(77);
        return array.length;
    }

    function fail() internal pure returns (uint256) {
        revert("fail");
    }

    function indexedArguments() public returns (uint256) {
        order = 0;
        emit Indexed(t(1), t(2));
        return order;
    }

    function indexedReference() public {
        uint256[] memory values = new uint256[](1);
        values[0] = 1;
        emit Hashed(values, mutate(values));
    }

    function copy() public {
        emit Copied(array, grow());
    }

    function indexedReferenceOrder() public {
        uint256[] memory values = new uint256[](1);
        values[0] = 1;
        emit Pair(mutate(values), values);
    }

    function indexedEnum() public {
        Choice dirty;
        assembly {
            dirty := 2
        }
        emit Chosen(fail(), dirty);
    }
}
