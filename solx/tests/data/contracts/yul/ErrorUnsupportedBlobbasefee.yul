object "ErrorUnsupportedBlobbasefee" {
    code {
        {
            let size := datasize("ErrorUnsupportedBlobbasefee_deployed")
            codecopy(0, dataoffset("ErrorUnsupportedBlobbasefee_deployed"), size)
            return(0, size)
        }
    }
    object "ErrorUnsupportedBlobbasefee_deployed" {
        code {
            {
                let fee := blobbasefee()
                return(0, 0)
            }
        }
    }
}
