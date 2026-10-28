// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Base {
    uint256 value;

    function set(uint256 x) external {
        value = x;
    }
}

contract Derived is Base {}
