//! This module contains the definition and implementation of [`LittleEndianBytes`], a trait to
//! convert values (such as integers or floats) to bytes in little endian byter order

/// This macro implements the [`LittleEndianBytes`] trait for a provided list of types.
///
/// # Assumptions
///
/// Each type for which this macro is executed must provide a `from_le_bytes` and `to_le_bytes`
/// function.
macro_rules! impl_LittleEndianBytes{
        [$($type:ty),+] => {

            $(impl LittleEndianBytes<{ ::core::mem::size_of::<$type>() }> for $type {
                fn from_le_bytes(bytes: [u8; ::core::mem::size_of::<$type>()]) -> Self {
                    Self::from_le_bytes(bytes)
                }

                fn to_le_bytes(self) -> [u8; ::core::mem::size_of::<$type>()] {
                    self.to_le_bytes()
                }
            })+
        }
    }

/// Convert from and to the little endian byte representation of a value
///
/// `N` denotes the number of bytes required for the little endian representation
pub trait LittleEndianBytes<const N: usize> {
    /// Convert from a byte array to Self
    fn from_le_bytes(bytes: [u8; N]) -> Self;

    /// Convert from self to a byte array
    fn to_le_bytes(self) -> [u8; N];
}

// implements the [`LittleEndianBytes`]
impl_LittleEndianBytes![i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64];
