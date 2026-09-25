object "Deploy" {
    code {
        {
            return(0, 0)
        }
    }

    object "Runtime" {
        code {
            {
                mstore(0, 42)
                return(0, 32)
            }
        }
    }
}
