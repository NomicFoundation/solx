// SPDX-License-Identifier: MIT

pragma solidity >=0.6.0;

import "./deploy.sol";

contract Main {
    function runtime(uint256 len) external {
        assembly {
            let result := call(gas(), address(), 0, 0, len, 32, 32)
            mstore(0, result)
            return(0, 64)
        }
    }

    function deploy(uint256 len) external {
        bytes memory deploy_calldata = type(Deploy).creationCode;
        assembly {
            // EIP-3860 caps init code at 49152 bytes; the largest case asks for more.
            let size := add(mload(deploy_calldata), len)
            if gt(size, 49152) { size := 49152 }
            let result := create(0, add(deploy_calldata, 32), size)
            if gt(result, 0) {
                result := 1
            }
            mstore(0, result)
            return(0, 32)
        }
    }

    fallback() external {
        assembly {
            mstore(0, calldatasize())
            return(0, 32)
        }
    }
}