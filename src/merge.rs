pub trait Merge {
    fn merge(&mut self, other: Self);
}

impl Merge for String {
    fn merge(&mut self, other: Self) {
        *self = other;
    }
}

impl Merge for u32 {
    fn merge(&mut self, other: Self) {
        *self = other;
    }
}

impl Merge for bool {
    fn merge(&mut self, other: Self) {
        *self = other;
    }
}
