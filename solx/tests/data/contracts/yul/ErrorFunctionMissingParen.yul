object "ErrorFunctionMissingParen" {
    code {
        {
            return(0, 0)
        }
    }
    object "ErrorFunctionMissingParen_deployed" {
        code {
            {
                return(0, 0)
            }

            function test{) -> result {
                result := 42
            }
        }
    }
}
