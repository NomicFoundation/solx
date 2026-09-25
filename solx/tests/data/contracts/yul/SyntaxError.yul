object "SyntaxError" {
    code {
        { 42 := 1 }
    }
    object "SyntaxError_deployed" {
        code {
            { return(0, 0) }
        }
    }
}
