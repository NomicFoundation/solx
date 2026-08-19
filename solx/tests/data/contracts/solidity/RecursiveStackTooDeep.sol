// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract RecursiveStackTooDeep {
    function f(
        uint256 a1, uint256 a2, uint256 a3, uint256 a4, uint256 a5, uint256 a6,
        uint256 a7, uint256 a8, uint256 a9, uint256 a10, uint256 a11, uint256 a12,
        uint256 a13, uint256 a14, uint256 a15, uint256 a16, uint256 a17, uint256 a18
    ) public returns (uint256) {
        if (a1 == 0) {
            return a2 + a17 + a18;
        }
        uint256 r = f(
            a1 - 1, a2 + 1, a3 ^ a4, a4 + a5, a5 * 2, a6 + a7, a7 ^ a8, a8 + 1,
            a9 + a10, a10 ^ a11, a11 + a12, a12 + 1, a13 ^ a14, a14 + a15,
            a15 + 1, a16 ^ a17, a17 + a18, a18 + a1
        );
        return r + a1 + a2 + a3 + a4 + a5 + a6 + a7 + a8 + a9 + a10 + a11
            + a12 + a13 + a14 + a15 + a16 + a17 + a18;
    }
}
