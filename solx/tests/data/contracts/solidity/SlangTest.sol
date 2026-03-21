// SPDX-License-Identifier: MIT

pragma solidity >=0.4.16;

contract SlangTest {
    uint256 public count;

    function add(uint256 a, uint256 b) public pure returns (uint256) {
        return a + b;
    }

    function compare(uint256 a, uint256 b) public pure returns (uint256) {
        if (a > b) {
            return a - b;
        } else {
            return b - a;
        }
    }

    function sum(uint8 n) public pure returns (uint256) {
        uint256 total = 0;
        for (uint8 i = 0; i < n; i++) {
            total += i;
        }
        return total;
    }

    function increment() public {
        count += 1;
    }

    function getCount() public view returns (uint256) {
        return count;
    }
}
