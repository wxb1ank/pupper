use std::convert::{TryFrom, TryInto as _};

pub struct Table<T>(Vec<T>);

impl<'a, T> TryFrom<&'a [u8]> for Table<T>
where
    T: super::LoadableRegion<'a>,
    [(); T::SIZE]: Sized,
{
    type Error = crate::Error;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        let entries = data.chunks_exact(T::SIZE)
            .flat_map(|x| <&[u8; T::SIZE]>::try_from(x).unwrap().try_into())
            .collect();

        Ok(Self(entries))
    }
}

impl<T> std::ops::Deref for Table<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
