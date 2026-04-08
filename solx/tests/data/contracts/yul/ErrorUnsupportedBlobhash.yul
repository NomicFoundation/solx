object "ErrorUnsupportedBlobhash" {
    code {
        {
            let size := datasize("ErrorUnsupportedBlobhash_deployed")
            codecopy(0, dataoffset("ErrorUnsupportedBlobhash_deployed"), size)
            return(0, size)
        }
    }
    object "ErrorUnsupportedBlobhash_deployed" {
        code {
            {
                let hash := blobhash(0)
                return(0, 0)
            }
        }
    }
}
