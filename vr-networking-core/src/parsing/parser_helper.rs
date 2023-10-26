
use thiserror::Error;

trait PacketComponent
where
    Self: Sized,
{
    fn serialize(&self) -> Box<[u8]>;
    fn deserialize<T: Iterator<Item = u8>>(a: T) -> Result<Self, PacketParserError>;
}



macro_rules! impl_number_packets_serde {
    {$($type: ident),*} => {
        $(impl PacketComponent for $type {
            fn serialize(&self) -> Box<[u8]> {
                self.to_be_bytes().into()
            }
        
            fn deserialize<T: Iterator<Item = u8>>(mut a: T) -> Result<Self, PacketParserError> {
                let mut array = [0u8; core::mem::size_of::<Self>()];
                for i in 0..core::mem::size_of::<Self>() {
                    array[i] = a.next().ok_or(PacketParserError::ComponentToShort)?
                }
                Ok(Self::from_be_bytes(array))
            }
        })*
    };
}

impl_number_packets_serde!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128);

#[derive(Debug, Clone, Copy, Error)]
enum PacketParserError {
    #[error("encountered packet length not matching expected length")]
    ComponentToShort,
}
