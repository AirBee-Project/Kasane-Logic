use serde::Serialize;
use std::fmt;

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dimension<T> {
    pub start: Option<T>,
    pub end: Option<T>,
}

impl<T> Dimension<T>
where
    T: PartialOrd + Clone,
{
    /// StartとEndの値が逆の場合は入れ替える
    pub fn new(start: Option<T>, end: Option<T>) -> Self
    where
        T: PartialOrd + Clone,
    {
        match (&start, &end) {
            (Some(s), Some(e)) if s > e => Self {
                start: end.clone(),
                end: start.clone(),
            },
            _ => Self { start, end },
        }
    }
}

impl<T> fmt::Display for Dimension<T>
where
    T: fmt::Display + PartialEq,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.start, &self.end) {
            (None, None) => write!(f, "-"),
            (None, Some(e)) => write!(f, "-:{}", e),
            (Some(s), None) => write!(f, "{}:-", s),
            (Some(s), Some(e)) => write!(f, "{}:{}", s, e),
        }
    }
}
