// SPDX-License-Identifier: MIT

pragma solidity >=0.8.4;

interface IERC20Minimal {
    function balanceOf(address account) external view returns (uint256);
}

contract Main {
    error transferFailed();

    address immutable token;

    constructor(address _token) {
        token = _token;
    }

    function _balance() internal view returns (uint256) {
        return IERC20Minimal(token).balanceOf(address(this));
    }

    function safeTransfer(address _token, address to, uint256 amount) internal returns(bool success) {
        assembly {
            let freeMemoryPointer := mload(0x40) // we will need to restore 0x40 slot
            mstore(0x00, 0xa9059cbb00000000000000000000000000000000000000000000000000000000) // "transfer(address,uint256)" selector
            mstore(0x04, and(to, 0xffffffffffffffffffffffffffffffffffffffff)) // append cleaned "to" address
            mstore(0x24, amount)
            // now we use 0x00 - 0x44 bytes (68), freeMemoryPointer is dirty
            success := and(
                // set success to true if call isn't reverted and returned exactly 1 (can't just be non-zero data) or nothing
                call(gas(), _token, 0, 0, 0x44, 0, 0x20),
                or(and(eq(mload(0), 1), eq(returndatasize(), 32)), iszero(returndatasize()))
            )
            mstore(0x40, freeMemoryPointer) // restore the freeMemoryPointer
        }

        if (!success) revert transferFailed();
    }

    function test() external returns(uint256 balance) {
        uint256 balance = _balance();
        safeTransfer(token, address(this), 10000);
    }
}
