object "ErrorBlockMissingBrace" {
    code {
        {
            if 1 42
        }
    }
    object "ErrorBlockMissingBrace_deployed" {
        code {
            { return(0, 0) }
        }
    }
}
