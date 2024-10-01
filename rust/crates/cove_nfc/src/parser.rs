use nom::{
    bits::{bits, streaming::take as take_bits},
    bytes::streaming::{tag, take},
    combinator::map,
    number::streaming::{be_u16, be_u32, be_u8},
    sequence::tuple,
    IResult,
};

#[derive(Debug)]
struct NdefHeader {
    message_begin: bool,
    message_end: bool,
    chunked: bool,
    short_record: bool,
    has_id_length: bool,
    type_name_format: u8,
    type_length: u8,
    payload_length: u32,
    id_length: Option<u8>,
}

#[derive(Debug, Clone)]
enum NdefType {
    Empty,
    WellKnown,
    Mime,
    AbsoluteUri,
    External,
    Unknown,
    Unchanged,
    Reserved,
}

#[derive(Debug)]
struct NdefRecord {
    header: NdefHeader,
    type_: Vec<u8>,
    id: Option<Vec<u8>>,
    payload: Vec<u8>,
}

fn parse_header(input: &[u8]) -> IResult<&[u8], NdefHeader> {
    let (
        input,
        (message_begin, message_end, chunk_flag, short_record, id_length_present, type_name_format),
    ) = bits::<_, _, nom::error::Error<(&[u8], usize)>, _, _>(map(
        tuple((
            take_bits(1usize),
            take_bits(1usize),
            take_bits(1usize),
            take_bits(1usize),
            take_bits(1usize),
            take_bits(3u8),
        )),
        |(a, b, c, d, e, f): (u8, u8, u8, u8, u8, u8)| (a == 1, b == 1, c == 1, d == 1, e == 1, f),
    ))(input)?;

    let (input, type_length) = be_u8(input)?;

    let (input, payload_length) = if short_record {
        map(be_u8, |x| x as u32)(input)?
    } else {
        be_u32(input)?
    };

    let (input, id_length) = if id_length_present {
        map(be_u8, Some)(input)?
    } else {
        (input, None)
    };

    Ok((
        input,
        NdefHeader {
            message_begin,
            message_end,
            chunked: chunk_flag,
            short_record,
            has_id_length: id_length_present,
            type_name_format,
            type_length,
            payload_length,
            id_length,
        },
    ))
}
fn parse_payload_length(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, _) = tag(&[226, 67, 0, 1, 0, 0, 4, 0, 3])(input)?;

    let (input, length_indicator) = be_u8(input)?;

    if length_indicator == 255 {
        let (input, payload_length) = be_u16(input)?;
        Ok((input, payload_length))
    } else {
        Ok((input, length_indicator as u16))
    }
}

fn parse_type(input: &[u8], type_length: u8) -> IResult<&[u8], Vec<u8>> {
    map(take(type_length), |s: &[u8]| s.to_vec())(input)
}

fn parse_id(input: &[u8], id_length: Option<u8>) -> IResult<&[u8], Option<Vec<u8>>> {
    if let Some(id_len) = id_length {
        map(take(id_len), |s: &[u8]| Some(s.to_vec()))(input)
    } else {
        Ok((input, None))
    }
}

fn parse_payload(input: &[u8], payload_length: u32) -> IResult<&[u8], Vec<u8>> {
    map(take(payload_length), |s: &[u8]| s.to_vec())(input)
}

fn parse_ndef_record(input: &[u8]) -> IResult<&[u8], NdefRecord> {
    let (input, header) = parse_header(input)?;
    let (input, type_) = parse_type(input, header.type_length)?;
    let (input, id) = parse_id(input, header.id_length)?;
    let (input, payload) = parse_payload(input, header.payload_length)?;

    Ok((
        input,
        NdefRecord {
            header,
            type_,
            id,
            payload,
        },
    ))
}

fn parse_ndef_message(input: &[u8]) -> IResult<&[u8], Vec<NdefRecord>> {
    nom::multi::many1(parse_ndef_record)(input)
}

fn main() {
    // Example usage with incomplete data
    let incomplete_ndef_data = vec![
        0xD1, 0x01, 0x0E, 0x55, 0x03, 0x67, 0x6F, 0x6F,
        // ... missing rest of the data
    ];

    match parse_ndef_message(&incomplete_ndef_data) {
        Ok((remaining, records)) => {
            println!("Parsed NDEF records: {:?}", records);
            println!("Remaining data: {:?}", remaining);
        }
        Err(nom::Err::Incomplete(needed)) => {
            println!("Need more data: {:?}", needed);
        }
        Err(e) => println!("Error: {:?}", e),
    }
}

fn process_stream(mut data: &[u8]) {
    loop {
        match parse_ndef_message(data) {
            Ok((remaining, records)) => {
                println!("Parsed records: {:?}", records);
                data = remaining;
            }
            Err(nom::Err::Incomplete(_)) => {
                println!("Need more data");
                break; // Wait for more data
            }
            Err(e) => {
                println!("Error: {:?}", e);
                break; // Handle error
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;

    static EXPORT: LazyLock<Vec<u8>> = LazyLock::new(|| {
        let file_contents = include_bytes!("../test/data/export_bytes.txt");
        let file_string = String::from_utf8(file_contents.to_vec()).unwrap();

        let numbers: Vec<u8> = file_string
            .split(',')
            .map(|s| s.trim())
            .map(|s| s.parse::<u8>().unwrap())
            .collect();

        numbers
    });

    static DESCRIPTOR: LazyLock<Vec<u8>> = LazyLock::new(|| {
        let file_contents = include_bytes!("../test/data/descriptor_bytes.txt");
        let file_string = String::from_utf8(file_contents.to_vec()).unwrap();

        let numbers: Vec<u8> = file_string
            .split(',')
            .map(|s| s.trim())
            .map(|s| s.parse::<u8>().unwrap())
            .collect();

        numbers
    });

    fn export_bytes() -> &'static [u8] {
        let (data, _payload_length) = parse_payload_length(&EXPORT).unwrap();
        data
    }

    fn descriptor_bytes() -> &'static [u8] {
        let (data, _payload_length) = parse_payload_length(&DESCRIPTOR).unwrap();
        data
    }

    #[test]
    fn known_header_parse() {
        let header_bytes = [0xD1, 0x01, 0x0D, 0x55, 0x02];
        let (_, header) = parse_header(&header_bytes).unwrap();
        assert!(header.message_begin);
        assert!(header.message_end);
        assert!(!header.chunked);
        assert!(header.short_record);
        assert!(!header.has_id_length);
        assert_eq!(header.type_name_format, 1);
        assert_eq!(header.type_length, 1);
        assert_eq!(header.payload_length, 13);
    }

    #[test]
    fn test_header_parsing_with_complete_data() {
        // export
        let (data, payload_length) = parse_payload_length(&EXPORT).unwrap();
        assert_eq!(payload_length, 3031);

        let (_, header) = parse_header(&data).unwrap();
        assert!(header.message_begin);
        assert!(header.message_end);
        assert!(!header.chunked);
        assert!(!header.short_record);
        assert!(!header.has_id_length);
        assert_eq!(header.type_name_format, 2);
        assert_eq!(header.type_length, 16);
        assert_eq!(header.payload_length, 3009);

        // descriptor
        let (data, payload_length) = parse_payload_length(&DESCRIPTOR).unwrap();
        assert_eq!(payload_length, 161);

        let (_, header) = parse_header(&data).unwrap();
        assert!(header.message_begin);
        assert!(header.message_end);
        assert!(!header.chunked);
        assert!(header.short_record);
        assert!(!header.has_id_length);
        assert_eq!(header.type_name_format, 1);
        assert_eq!(header.type_length, 1);
        assert_eq!(header.payload_length, 157)
    }

    #[test]
    fn test_header_parsing_with_incomplete_data() {
        let data = descriptor_bytes();
        let (_, header) = parse_header(&data[0..6]).unwrap();
        assert!(header.message_end);
        assert!(header.message_begin);
        assert!(header.message_end);
        assert!(!header.chunked);
        assert!(header.short_record);
        assert!(!header.has_id_length);
        assert_eq!(header.type_name_format, 1);
        assert_eq!(header.type_length, 1);
        assert_eq!(header.payload_length, 157)
    }

    #[test]
    fn parse_type_with_complete_data() {
        // export
        let data = export_bytes();
        let (data, header) = parse_header(&data).unwrap();
        let (_, type_) = parse_type(data, header.type_length).unwrap();
        let type_string = String::from_utf8(type_).unwrap();
        assert_eq!(type_string, "application/json");
    }
    //
    #[test]
    fn parse_payload_with_complete_data_descriptor() {
        // let record_1
        let data = descriptor_bytes();
        let (data, header) = parse_header(&data).unwrap();
        let (data, type_) = parse_type(data, header.type_length).unwrap();
        let (data, id) = parse_id(data, header.id_length).unwrap();
        let (_data, payload) = parse_payload(data, header.payload_length).unwrap();

        println!("{:?}", &payload[..4]);

        let type_string = String::from_utf8(type_).unwrap();
        assert_eq!(type_string, "T".to_string());
        assert_eq!(id, None);

        let payload_string = String::from_utf8(payload).unwrap();
        let descriptor_string = std::fs::read_to_string("test/data/descriptor.txt").unwrap();

        // assert_eq!(payload_string, descriptor_string);
    }
    //
    #[test]
    fn parse_payload_with_complete_data_export() {
        let data = export_bytes();
        let (data, header) = parse_header(&data).unwrap();
        let (data, _type) = parse_type(data, header.type_length).unwrap();
        let (data, _id) = parse_id(data, header.id_length).unwrap();
        let (_data, payload) = parse_payload(data, header.payload_length).unwrap();

        let payload_string = String::from_utf8(payload).unwrap();
        let export_string = std::fs::read_to_string("test/data/export.json").unwrap();

        let payload_json = serde_json::from_str::<serde_json::Value>(&payload_string).unwrap();
        let export_json = serde_json::from_str::<serde_json::Value>(&export_string).unwrap();

        assert_eq!(payload_json, export_json);
    }
}
