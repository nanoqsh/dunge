use std::iter;

pub fn eq_lines<A, B>(a: A, b: B)
where
    A: AsRef<str>,
    B: AsRef<str>,
{
    for (x, y) in iter::zip(a.as_ref().lines(), b.as_ref().lines()) {
        assert_eq!(x, y, "lines should be equal");
    }
}
