object "ErrorFunctionInvalidAttrsRepeated" {
    code {
        {
            return(0, 0)
        }
    }
    object "ErrorFunctionInvalidAttrsRepeated_deployed" {
        code {
            {
                return(0, 0)
            }

            function test_$llvm_UnknownAttribute1_UnknownAttribute1_UnknownAttribute2_llvm$_test() -> result {
                result := 42
            }
        }
    }
}
