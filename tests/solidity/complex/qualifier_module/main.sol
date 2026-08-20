// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

import "./module.sol" as M;
import "./module.sol" as Second;
import * as Star from "./module.sol";

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

    function constantMember() public returns (uint256, uint256) {
        sequence = 0;
        uint256 value = (mark(1) ? M : M).FREE_K;
        return (value, sequence);
    }

    function freeCall() public returns (uint256, uint256) {
        sequence = 0;
        uint256 value = (mark(1) ? M : M).half(eight());
        return (value, sequence);
    }

    function designator() public returns (uint256, uint256) {
        sequence = 0;
        uint256 value = ((mark(1) ? M : M).half)(eight());
        return (value, sequence);
    }

    function chain() public returns (uint256, uint256) {
        sequence = 0;
        uint256 value = (mark(1) ? M : M).Nested.FREE_K;
        return (value, sequence);
    }

    function aliased() public returns (uint256, uint256) {
        sequence = 0;
        uint256 value = (mark(1) ? M : Second).FREE_K;
        return (value, sequence);
    }

    function stray() public returns (uint256) {
        sequence = 0;
        (mark(1) ? M : M).Test;
        return sequence;
    }

    function plain() public pure returns (uint256) {
        return M.half(8);
    }

    function chained() public pure returns (uint256) {
        return M.Mod.half(8);
    }

    function starred() public pure returns (uint256) {
        return Star.half(8);
    }
}
