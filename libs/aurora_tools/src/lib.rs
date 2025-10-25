pub fn lib_name() -> &'static str {
    "aurora-tools"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_test() {
        let result = lib_name();
        assert_eq!(result, "aurora-tools");
    }
}
