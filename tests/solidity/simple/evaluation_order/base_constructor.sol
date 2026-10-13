//! { "modes": [ "E" ], "cases": [ {
//!     "name": "arguments",
//!     "inputs": [ { "method": "order", "calldata": [] } ],
//!     "expected": [ "21" ]
//! } ] }

// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

abstract contract Recorder {
    uint256 public order;

    function t(uint256 n) internal returns (uint256) {
        order = order * 10 + n;
        return n;
    }
}

abstract contract First is Recorder {
    constructor(uint256) {}
}

abstract contract Second is Recorder {
    constructor(uint256) {}
}

abstract contract Middle is Second {
    constructor() Second(t(2)) {}
}

contract Test is First, Middle {
    constructor() First(t(1)) Middle() {}
}
