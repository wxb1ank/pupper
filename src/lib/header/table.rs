use std::convert::{TryFrom, TryInto as _};

use crate::FixedSize;

#[derive(Clone, Default)]
pub struct Table<T>(pub Vec<T>);

impl<'a, T: FixedSize + TryFrom<&'a [u8; T::SIZE]>> TryFrom<&'a [u8]> for Table<T>
where
    [(); T::SIZE]: Sized,
{
    type Error = crate::Error;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let entries = data
            .chunks_exact(T::SIZE)
            .flat_map(|x| <&[u8; T::SIZE]>::try_from(x).unwrap().try_into())
            .collect();

        Ok(Self(entries))
    }
}

impl<'a, T: FixedSize + Copy> From<&Table<T>> for Vec<u8>
where
    [u8; T::SIZE]: From<T>,
{
    fn from(table: &Table<T>) -> Self {
        table
            .0
            .iter()
            .flat_map(|x| <[u8; T::SIZE]>::from(*x).to_vec())
            .collect()
    }
}

impl<T> std::ops::Deref for Table<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
