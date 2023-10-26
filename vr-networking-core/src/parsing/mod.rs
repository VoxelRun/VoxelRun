use thiserror::Error;

use self::packet::Packet;

mod packet;
mod parser_helper;

#[repr(C)]
#[derive(PartialEq, Eq, Debug)]
struct ParserPacket {
    ptr: *mut u8,
    len: usize,
}

type Parser = extern "C" fn(packet: ParserPacket) -> Response;

#[repr(C)]
struct PacketParser {
    parser_types: Vec<Parser>,
}

impl PacketParser {
    fn new() -> Self {
        PacketParser {
            parser_types: vec![],
        }
    }

    #[export_name = "register_parser"]
    extern "C" fn register_parser(&mut self, parser: Parser) -> u32 {
        if self.parser_types.len() == u32::MAX as usize {
            panic!("Exceeds Parser limit")
        }
        self.parser_types.push(parser);
        (self.parser_types.len() - 1) as u32
    }

    fn parse_packet(&self, packet: Packet) -> Result<Response, PacketError> {
        let packet_id = packet.get_packet_id()?;
        let Some(parser) = self.parser_types.get(packet_id as usize) else {
            return Err(PacketError::IdDoesNotExist(packet_id));
        };
        Ok(parser(packet.try_into()?))
    }
}

#[derive(PartialEq, Eq, Debug)]
#[repr(C)]
enum Response {
    X,
}

#[derive(Error, Debug, PartialEq, Eq)]
enum PacketError {
    #[error("packet is to short to contain an id")]
    PacketHasNoId,
    #[error("packet with id {0} does not exist. This is indicative of a client-server desync")]
    IdDoesNotExist(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_parser() {
        extern "C" fn parser_fn(packet: ParserPacket) -> Response {
            println!("hi");
            Response::X
        }
        let mut packet_parser = PacketParser::new();
        assert_eq!(packet_parser.register_parser(parser_fn), 0);
        assert_eq!(packet_parser.register_parser(parser_fn), 1);
        assert_eq!(packet_parser.parser_types[0] as usize, parser_fn as usize);
        assert_eq!(packet_parser.parser_types[1] as usize, parser_fn as usize);
    }

    #[test]
    fn parse_packet_not_exist() {
        let packet_parser = PacketParser::new();
        let result = 2u32;
        let test_packet = Packet {
            ptr: &mut [
                ((result & 0xff000000) >> 24) as u8,
                ((result & 0x00ff0000) >> 16) as u8,
                ((result & 0x0000ff00) >> 8) as u8,
                (result & 0x000000ff) as u8,
            ] as *mut u8,
            len: 4,
        };
        assert_eq!(
            Err(PacketError::IdDoesNotExist(result)),
            packet_parser.parse_packet(test_packet)
        );
    }

    #[test]
    fn parse_packet_exist() {
        extern "C" fn parser_fn(packet: ParserPacket) -> Response {
            println!("hi");
            Response::X
        }
        let mut packet_parser = PacketParser::new();
        packet_parser.register_parser(parser_fn);
        let result = 0u32;
        let test_packet = Packet {
            ptr: &mut [
                ((result & 0xff000000) >> 24) as u8,
                ((result & 0x00ff0000) >> 16) as u8,
                ((result & 0x0000ff00) >> 8) as u8,
                (result & 0x000000ff) as u8,
            ] as *mut u8,
            len: 4,
        };
        assert_eq!(Ok(Response::X), packet_parser.parse_packet(test_packet));
    }
}
