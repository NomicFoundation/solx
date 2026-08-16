// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

import "./library.sol" as M;

contract Test {
    uint256 sequence;

    function mark(uint256 digit) internal returns (bool) {
        sequence = sequence * 10 + digit;
        return true;
    }

    function eight() internal returns (uint256) {
        mark(2);
        return 8;
    }

    function externalCall() public returns (uint256, uint256) {
        sequence = 0;
        uint256 value = (mark(1) ? M : M).Halver.half(eight());
        return (value, sequence);
    }

    function tried() public returns (uint256, uint256) {
        sequence = 0;
        try (mark(1) ? M : M).Halver.half(eight()) returns (uint256 value) {
            return (value, sequence);
        } catch {
            return (0, sequence);
        }
    }
}
