// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

import "./main.sol";

enum Tier {
    Low,
    High
}

interface Surface {
    enum Level {
        Low,
        High
    }
}

type Cost is uint256;

struct Pair {
    uint256 first;
}

library Halver {
    uint256 internal constant SEVEN = 7;

    function half(uint256 x) internal pure returns (uint256) {
        return x / 2;
    }
}
