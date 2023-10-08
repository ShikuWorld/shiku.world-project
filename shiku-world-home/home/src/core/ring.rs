pub struct Cycle<T: Sized, const N: usize> {
    data: [T; N],
    i: usize,
}

impl<T: Sized, const N: usize> Cycle<T, N> {
    pub fn new(data: [T; N]) -> Cycle<T, N> {
        Cycle { i: 0, data }
    }

    pub fn next(&mut self) -> Option<&T> {
        let next_data = &self.data[self.i];
        self.i = (self.i + 1) % N;
        Some(next_data)
    }
}
