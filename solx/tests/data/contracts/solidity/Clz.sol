// SPDX-License-Identifier: MIT

pragma solidity >=0.8.31;

contract Clz {
    function f(uint256 v, uint256 n) external pure returns (uint256) {
        return g(v, n);
    }

    function g(uint256 v, uint256 n) internal pure returns (uint256 r) {
        if (n == 0) {
            assembly {
                r := clz(v)
            }
            return r;
        }
        return g(v >> 1, n - 1) + g(v, n - 1);
    }
}
