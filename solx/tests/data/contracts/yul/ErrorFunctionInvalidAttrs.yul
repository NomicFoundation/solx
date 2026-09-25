object "ErrorFunctionInvalidAttrs" {
    code {
        {
            return(0, 0)
        }
    }
    object "ErrorFunctionInvalidAttrs_deployed" {
        code {
            {
                return(0, 0)
            }

            function test_$llvm_UnknownAttribute_llvm$_test() -> result {
                result := 42
            }
        }
    }
}
