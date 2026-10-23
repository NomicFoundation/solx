// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract InternalFunctionPointer {
    function(uint256) internal pure returns (uint256) stored;

    constructor() {
        stored = double;
    }

    function choose(bool increments) external {
        stored = increments ? increment : double;
    }

    function run(uint256 value) external view returns (uint256) {
        return stored(value);
    }

    function increment(uint256 value) internal pure returns (uint256) {
        return value + 1;
    }

    function double(uint256 value) internal pure returns (uint256) {
        return value * 2;
    }
}
