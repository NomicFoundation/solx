object "ErrorFunctionMissingCloseParen" {
    code {
        {
            return(0, 0)
        }
    }
    object "ErrorFunctionMissingCloseParen_deployed" {
        code {
            {
                return(0, 0)
            }

            function test(} -> result {
                result := 42
            }
        }
    }
}
