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

    function seven() internal pure returns (uint256) {
        return 7;
    }

    function enumMember() public pure returns (uint256) {
        return uint256(M.Tier.High);
    }

    function interfaceEnum() public pure returns (uint256) {
        return uint256(M.Surface.Level.High);
    }

    function stateRead() public returns (uint256) {
        stored = 9;
        return M.Test.stored;
    }

    function stateWrite() public returns (uint256) {
        M.Test.stored = 9;
        return stored;
    }

    function stateCompound() public returns (uint256) {
        stored = 9;
        M.Test.stored += 9;
        return stored;
    }

    function stateDelete() public returns (uint256) {
        stored = 9;
        delete M.Test.stored;
        return stored;
    }

    function fieldRead() public returns (uint256) {
        inner.value = 9;
        return M.Test.inner.value;
    }

    function fieldWrite() public returns (uint256) {
        M.Test.inner.value = 9;
        return inner.value;
    }

    function constantMember() public pure returns (uint256) {
        return M.Test.SEVEN;
    }

    function immutableMember() public view returns (uint256) {
        return M.Test.given;
    }

    function internalCall() public pure returns (uint256) {
        return M.Test.seven();
    }

    function libraryCall() public pure returns (uint256) {
        return M.Halver.half(8);
    }

    function libraryConstant() public pure returns (uint256) {
        return M.Halver.SEVEN;
    }

    function wrap() public pure returns (uint256) {
        return M.Cost.unwrap(M.Cost.wrap(9));
    }

    function construction() public pure returns (uint256) {
        M.Pair memory pair = M.Pair(9);
        return pair.first;
    }
}
