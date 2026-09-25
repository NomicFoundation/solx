object "ExternalCalls" {
    code {
        {
            let size := datasize("ExternalCalls_deployed")
            codecopy(0, dataoffset("ExternalCalls_deployed"), size)
            return(0, size)
        }
    }
    object "ExternalCalls_deployed" {
        code {
            {
                let target := caller()
                let g := gas()

                // call(gas, addr, value, inOffset, inSize, outOffset, outSize)
                let r1 := call(g, target, 0, 0, 0, 0x100, 32)

                // staticcall(gas, addr, inOffset, inSize, outOffset, outSize)
                let r2 := staticcall(g, target, 0, 0, 0x120, 32)

                // delegatecall(gas, addr, inOffset, inSize, outOffset, outSize)
                let r3 := delegatecall(g, target, 0, 0, 0x140, 32)

                // create(value, offset, size)
                mstore(0, 0x600160005260206000F3)
                let newAddr := create(0, 0, 32)

                // create2(value, offset, size, salt)
                let newAddr2 := create2(0, 0, 32, 0x1234)

                // extcodecopy(addr, destOffset, offset, size)
                extcodecopy(target, 0x200, 0, 32)

                // returndatacopy (after a call)
                returndatacopy(0x300, 0, returndatasize())

                // Store results and return
                mstore(0, add(add(r1, r2), r3))
                return(0, 32)
            }
        }
    }
}
