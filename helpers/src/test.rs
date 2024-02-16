pub fn eq_lines(a: &str, b: &str) {
    for (x, y) in a.lines().zip(b.lines()) {
        assert_eq!(x, y, "lines should be equal");
    }
}
