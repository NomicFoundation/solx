// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

import "./main.sol";
import "./nested.sol";
import "./nested.sol" as Nested;

function half(uint256 x) pure returns (uint256) {
    return x / 2;
}
