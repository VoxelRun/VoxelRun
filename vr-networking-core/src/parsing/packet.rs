use super::{PacketError, ParserPacket};
pub const U32_SIZE: usize = core::mem::size_of::<u32>();

#[repr(C)]
pub(super) struct Packet {
    pub(super) ptr: *mut u8,
    pub(super) len: usize,
}

impl Packet {
    unsafe fn get_packet_id_unchecked(&self) -> u32 {
        let mut id = 0u32;
        for i in 0..U32_SIZE {
            id |= (*self.ptr.offset(i as isize) as u32) << 8 * (U32_SIZE - 1 - i)
        }
        id
    }

    pub(super) fn get_packet_id(&self) -> Result<u32, PacketError> {
        if self.len < U32_SIZE {
            return Err(PacketError::PacketHasNoId);
        }
        unsafe { Ok(self.get_packet_id_unchecked()) }
    }
}

impl TryInto<ParserPacket> for Packet {
    type Error = PacketError;

    fn try_into(self) -> Result<ParserPacket, Self::Error> {
        if self.len < U32_SIZE {
            return Err(PacketError::PacketHasNoId);
        }
        Ok(ParserPacket {
            ptr: unsafe { self.ptr.offset(U32_SIZE as isize) },
            len: self.len - U32_SIZE,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::parsing::ParserPacket;

    use super::{Packet, PacketError};

    #[test]
    fn get_packet_id_longer() {
        let result = 0x01020304u32;
        let test_packet = Packet {
            ptr: &mut [
                ((result & 0xff000000) >> 24) as u8,
                ((result & 0x00ff0000) >> 16) as u8,
                ((result & 0x0000ff00) >> 8) as u8,
                (result & 0x000000ff) as u8,
                7u8,
            ] as *mut u8,
            len: 5,
        };
        assert_eq!(result, test_packet.get_packet_id().unwrap())
    }

    #[test]
    fn get_packet_id_precise() {
        let result = 0x01020304u32;
        let test_packet = Packet {
            ptr: &mut [
                ((result & 0xff000000) >> 24) as u8,
                ((result & 0x00ff0000) >> 16) as u8,
                ((result & 0x0000ff00) >> 8) as u8,
                (result & 0x000000ff) as u8,
            ] as *mut u8,
            len: 4,
        };
        assert_eq!(result, test_packet.get_packet_id().unwrap())
    }

    #[test]
    fn get_packet_id_shorter() {
        let result = 0x01020304u32;
        let test_packet = Packet {
            ptr: &mut [
                ((result & 0xff000000) >> 24) as u8,
                ((result & 0x00ff0000) >> 16) as u8,
                ((result & 0x0000ff00) >> 8) as u8,
            ] as *mut u8,
            len: 3,
        };
        assert_eq!(Err(PacketError::PacketHasNoId), test_packet.get_packet_id())
    }

    #[test]
    fn try_into_parser_packet_longer() {
        let result = 0x01020304u32;
        let test_packet = Packet {
            ptr: &mut [
                ((result & 0xff000000) >> 24) as u8,
                ((result & 0x00ff0000) >> 16) as u8,
                ((result & 0x0000ff00) >> 8) as u8,
                (result & 0x000000ff) as u8,
                7u8,
            ] as *mut u8,
            len: 5,
        };
        let reference = ParserPacket {
            ptr: unsafe { test_packet.ptr.offset(4) },
            len: 1,
        };
        let parsed: ParserPacket = test_packet.try_into().unwrap();
        assert_eq!(reference, parsed)
    }

    #[test]
    fn try_into_parser_packet_precise() {
        let result = 0x01020304u32;
        let test_packet = Packet {
            ptr: &mut [
                ((result & 0xff000000) >> 24) as u8,
                ((result & 0x00ff0000) >> 16) as u8,
                ((result & 0x0000ff00) >> 8) as u8,
                (result & 0x000000ff) as u8,
            ] as *mut u8,
            len: 4,
        };
        assert_eq!(result, test_packet.get_packet_id().unwrap())
    }

    #[test]
    fn try_into_parser_packet_to_short() {
        let result = 0x01020304u32;
        let test_packet = Packet {
            ptr: &mut [
                ((result & 0xff000000) >> 24) as u8,
                ((result & 0x00ff0000) >> 16) as u8,
                ((result & 0x0000ff00) >> 8) as u8,
            ] as *mut u8,
            len: 3,
        };
        assert_eq!(
            Err(PacketError::PacketHasNoId),
            TryInto::<ParserPacket>::try_into(test_packet)
        )
    }
}
