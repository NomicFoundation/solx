object "Test" {
    code {
        {
            mstore(64, 128)
            if callvalue() { revert(0, 0) }
            let _1 := datasize("Test_deployed")
            codecopy(0, dataoffset("Test_deployed"), _1)
            return(0, _1)
        }
    }
    object "Test_deployed" {
        code {
            {
                mstore(64, 128)
                if iszero(lt(calldatasize(), 4))
                {
                    let _1 := 0
                    switch shr(224, calldataload(_1))
                    case 0x3df4ddf4 {
                        if callvalue() { revert(_1, _1) }
                        if slt(add(calldatasize(), not(3)), _1) { revert(_1, _1) }
                        let memPos := allocate_memory(_1)
                        mstore(memPos, 0x2a)
                        return(memPos, 32)
                    }
                    case 0x5a8ac02d {
                        if callvalue() { revert(_1, _1) }
                        if slt(add(calldatasize(), not(3)), _1) { revert(_1, _1) }
                        let memPos_1 := allocate_memory(_1)
                        return(memPos_1, sub(abi_encode_uint256(memPos_1, 0x63), memPos_1))
                    }
                }
                revert(0, 0)
            }
            function abi_encode_uint256(headStart, value0) -> tail
            {
                tail := add(headStart, 32)
                mstore(headStart, value0)
            }
            function allocate_memory(size) -> memPtr
            {
                memPtr := mload(64)
                let newFreePtr := add(memPtr, and(add(size, 31), not(31)))
                if or(gt(newFreePtr, 0xffffffffffffffff), lt(newFreePtr, memPtr))
                {
                    mstore(0, shl(224, 0x4e487b71))
                    mstore(4, 0x41)
                    revert(0, 0x24)
                }
                mstore(64, newFreePtr)
            }
        }
    }
}
