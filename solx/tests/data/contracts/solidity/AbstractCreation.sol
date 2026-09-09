// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

abstract contract Abstract {}

contract Creator {
    function create() public returns (Abstract) {
        return new Abstract();
    }
}
