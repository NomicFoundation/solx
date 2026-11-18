// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract Child {}

contract Parent {
    address public child;

    constructor() {
        child = address(new Child());
    }
}
