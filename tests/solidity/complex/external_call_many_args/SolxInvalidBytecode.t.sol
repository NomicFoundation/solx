// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.19;

import {PlaceholderContract} from "./Repro.sol";

contract SolxInvalidBytecodeTest {
    function testCheckEntrypointDoesNotHitInvalidBytecode() public returns (bool) {
        PlaceholderContract target = new PlaceholderContract();

        return target.check_entrypoint(
            false, false, false, false, false, false, false, false, false, false, false, false, false, false, false
        );
    }
}
