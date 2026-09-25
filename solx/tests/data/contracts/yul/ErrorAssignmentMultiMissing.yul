object "ErrorAssignmentMultiMissing" {
    code {
        {
            let a := 0
            let b := 0
            a, b {
        }
    }
    object "ErrorAssignmentMultiMissing_deployed" {
        code {
            { return(0, 0) }
        }
    }
}
