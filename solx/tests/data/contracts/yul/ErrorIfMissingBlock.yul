object "ErrorIfMissingBlock" {
    code {
        {
            if 1 42
        }
    }
    object "ErrorIfMissingBlock_deployed" {
        code {
            { return(0, 0) }
        }
    }
}
