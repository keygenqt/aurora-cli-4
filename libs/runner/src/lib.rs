pub fn lib_name() -> &'static str {
    "runner"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_test() {
        let result = lib_name();
        assert_eq!(result, "runner");
    }
}
