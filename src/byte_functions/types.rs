// Define a struct to wrap [u8; _] values
pub struct RawVal<const T: usize>(pub [u8; T]);

impl<const T: usize> std::ops::Index<usize> for RawVal<T> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        return &self.0[index];
    }
}
