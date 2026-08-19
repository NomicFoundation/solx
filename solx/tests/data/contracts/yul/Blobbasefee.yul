object "Blobbasefee" {
    code {
        {
            let size := datasize("Blobbasefee_deployed")
            codecopy(0, dataoffset("Blobbasefee_deployed"), size)
            return(0, size)
        }
    }
    object "Blobbasefee_deployed" {
        code {
            {
                mstore(0, blobbasefee())
                return(0, 32)
            }
        }
    }
}
