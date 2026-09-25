object "Test" {
    code {
        {
            return(0, 0)
        }
    }

    object "Test_deployed" {
        code {
            {
                mdelete(0, 42)
                return(0, 32)
            }
        }
    }
}
