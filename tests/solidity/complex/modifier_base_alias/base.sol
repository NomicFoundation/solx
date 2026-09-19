// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Base {
    uint256 seed;

    constructor(uint256 x) {
        seed = x;
    }

    function planted() public view returns (uint256) {
        return seed;
    }
}
