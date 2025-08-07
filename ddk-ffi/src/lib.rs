uniffi::include_scaffolding!("ddk_ffi");

pub fn hello_world() -> String {
    "Hello, World from Rust!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let result = hello_world();
        assert_eq!(result, "Hello, World from Rust!");
    }
}
