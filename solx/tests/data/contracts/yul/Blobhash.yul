object "Blobhash" {
    code {
        {
            let size := datasize("Blobhash_deployed")
            codecopy(0, dataoffset("Blobhash_deployed"), size)
            return(0, size)
        }
    }
    object "Blobhash_deployed" {
        code {
            {
                mstore(0, blobhash(0))
                return(0, 32)
            }
        }
    }
}
