// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

contract PayableConstructor {
    constructor() payable {}

    function one() public pure returns (uint256) {
        return 1;
    }
}
