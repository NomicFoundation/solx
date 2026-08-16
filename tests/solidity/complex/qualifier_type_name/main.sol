// SPDX-License-Identifier: MIT

pragma solidity >=0.8.0;

import "./module.sol" as M;

contract Test {
    struct Inner {
        uint256 value;
    }

    uint256 constant SEVEN = 7;
    uint256 immutable given = 9;
    uint256 stored;
    Inner inner;
    uint256 sequence;

    function mark(uint256 digit) internal returns (bool) {
        sequence = sequence * 10 + digit;
        return true;
    }

    function seven() internal returns (uint256) {
        mark(2);
        return 7;
    }

    function eight() internal returns (uint256) {
        mark(2);
        return 8;
    }

    function nine() internal returns (uint256) {
        mark(2);
        return 9;
    }

    function enumMember() public returns (uint256, uint256) {
        sequence = 0;
        M.Tier tier = (mark(1) ? M : M).Tier.High;
        return (uint256(tier), sequence);
    }

    function interfaceEnum() public returns (uint256, uint256) {
        sequence = 0;
        M.Surface.Level level = (mark(1) ? M : M).Surface.Level.High;
        return (uint256(level), sequence);
    }

    function stateRead() public returns (uint256, uint256) {
        stored = 9;
        sequence = 0;
        uint256 value = (mark(1) ? M : M).Test.stored;
        return (value, sequence);
    }

    function stateWrite() public returns (uint256, uint256) {
        sequence = 0;
        (mark(1) ? M : M).Test.stored = nine();
        return (stored, sequence);
    }

    function stateCompound() public returns (uint256, uint256) {
        stored = 9;
        sequence = 0;
        (mark(1) ? M : M).Test.stored += nine();
        return (stored, sequence);
    }

    function stateDelete() public returns (uint256, uint256) {
        stored = 9;
        sequence = 0;
        delete (mark(1) ? M : M).Test.stored;
        return (stored, sequence);
    }

    function fieldRead() public returns (uint256, uint256) {
        inner.value = 9;
        sequence = 0;
        uint256 value = (mark(1) ? M : M).Test.inner.value;
        return (value, sequence);
    }

    function fieldWrite() public returns (uint256, uint256) {
        sequence = 0;
        (mark(1) ? M : M).Test.inner.value = nine();
        return (inner.value, sequence);
    }

    function constantMember() public returns (uint256, uint256) {
        sequence = 0;
        uint256 value = (mark(1) ? M : M).Test.SEVEN;
        return (value, sequence);
    }

    function immutableMember() public returns (uint256, uint256) {
        sequence = 0;
        uint256 value = (mark(1) ? M : M).Test.given;
        return (value, sequence);
    }

    function internalCall() public returns (uint256, uint256) {
        sequence = 0;
        uint256 value = (mark(1) ? M : M).Test.seven();
        return (value, sequence);
    }

    function libraryCall() public returns (uint256, uint256) {
        sequence = 0;
        uint256 value = (mark(1) ? M : M).Halver.half(eight());
        return (value, sequence);
    }

    function libraryConstant() public returns (uint256, uint256) {
        sequence = 0;
        uint256 value = (mark(1) ? M : M).Halver.SEVEN;
        return (value, sequence);
    }

    function wrap() public returns (uint256, uint256) {
        sequence = 0;
        uint256 value = M.Cost.unwrap((mark(1) ? M : M).Cost.wrap(nine()));
        return (value, sequence);
    }

    function construction() public returns (uint256, uint256) {
        sequence = 0;
        M.Pair memory pair = (mark(1) ? M : M).Pair(nine());
        return (pair.first, sequence);
    }
}
