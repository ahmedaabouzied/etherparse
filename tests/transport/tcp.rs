use super::super::*;

use std::io::Cursor;
use proptest::prelude::*;
use self::byteorder::{ByteOrder, BigEndian};

#[test]
fn options() {
    let base : TcpHeader = Default::default();

    let dummy = [ 1, 2, 3, 4, 5,
                  6, 7, 8, 9,10,
                 11,12,13,14,15,
                 16,17,18,19,20,
                 21,22,23,24,25,
                 26,27,28,29,30,
                 31,32,33,34,35,
                 36,37,38,39,40,
                 41 ];

    //ok size -> expect output based on options size
    for i in 0..40 {
        let mut header = base.clone();
        println!("{}",i);
        //set the options
        header.set_options_raw(&dummy[..i]).unwrap();

        //determine the expected options length
        let mut options_length = i / 4;
        if i % 4 != 0 {
            options_length += 1;
        }
        options_length = options_length * 4;

        //expecetd data
        let mut expected_options = [0;40];
        expected_options[..i].copy_from_slice(&dummy[..i]);

        assert_eq!(options_length, header.options_len());
        assert_eq!((options_length / 4) as u8 + TCP_MINIMUM_DATA_OFFSET, header.data_offset());
        assert_eq!(&expected_options[..options_length], header.options());
    }

    //too big -> expect error
    let mut header = base.clone();
    use TcpOptionWriteError::*;
    assert_eq!(Err(NotEnoughSpace(dummy.len())), header.set_options_raw(&dummy[..]));
}

fn write_options(elements: &[TcpOptionElement]) -> TcpHeader {
    let mut result : TcpHeader = Default::default();
    result.set_options(elements).unwrap();
    result
}

proptest! {
    #[test]
    fn set_options_maximum_segment_size(arg in any::<u16>()) {
        use TcpOptionElement::*;
        assert_eq!(write_options(&[Nop, Nop, MaximumSegmentSize(arg), Nop]).options(), 
           &{
                let mut options = [
                    TCP_OPTION_ID_NOP, TCP_OPTION_ID_NOP, TCP_OPTION_ID_MAXIMUM_SEGMENT_SIZE, 4,
                    0, 0, TCP_OPTION_ID_NOP, TCP_OPTION_ID_END
                ];
                BigEndian::write_u16(&mut options[4..6], arg);
                options
            }
        );
    }
}

proptest! {
    #[test]
    fn set_options_window_scale(arg in any::<u8>()) {
        use TcpOptionElement::*;
        assert_eq!(write_options(&[Nop, Nop, WindowScale(arg), Nop]).options(), 
           &[
                TCP_OPTION_ID_NOP, TCP_OPTION_ID_NOP, TCP_OPTION_ID_WINDOW_SCALE, 3,
                arg, TCP_OPTION_ID_NOP, TCP_OPTION_ID_END, 0
            ]
        );
    }
}

#[test]
fn set_options_selective_ack_perm() {
    use TcpOptionElement::*;
    assert_eq!(write_options(&[Nop, Nop, SelectiveAcknowledgementPermitted, Nop]).options(), 
       &[
            TCP_OPTION_ID_NOP, TCP_OPTION_ID_NOP, TCP_OPTION_ID_SELECTIVE_ACK_PERMITTED, 2,
            TCP_OPTION_ID_NOP, TCP_OPTION_ID_END, 0, 0
        ]
    );
}

proptest! {
    #[test]
    fn set_options_selective_ack(args in proptest::collection::vec(any::<u32>(), 4*2)) {
        use TcpOptionElement::*;
        //1
        assert_eq!(write_options(&[Nop, Nop, SelectiveAcknowledgement((args[0], args[1]), [None, None, None]), Nop]).options(), 
           &{
                let mut options = [
                    TCP_OPTION_ID_NOP, TCP_OPTION_ID_NOP, TCP_OPTION_ID_SELECTIVE_ACK, 10,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    TCP_OPTION_ID_NOP, TCP_OPTION_ID_END, 0, 0
                ];
                BigEndian::write_u32(&mut options[4..8], args[0]);
                BigEndian::write_u32(&mut options[8..12], args[1]);
                options
            }
        );

        //2
        assert_eq!(write_options(&[Nop, Nop, SelectiveAcknowledgement((args[0], args[1]), 
                                                                      [Some((args[2], args[3])), 
                                                                       None, None]), 
                                   Nop]).options(), 
           &{
                let mut options = [
                    TCP_OPTION_ID_NOP, TCP_OPTION_ID_NOP, TCP_OPTION_ID_SELECTIVE_ACK, 18,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    TCP_OPTION_ID_NOP, TCP_OPTION_ID_END, 0, 0
                ];
                BigEndian::write_u32(&mut options[4..8], args[0]);
                BigEndian::write_u32(&mut options[8..12], args[1]);
                BigEndian::write_u32(&mut options[12..16], args[2]);
                BigEndian::write_u32(&mut options[16..20], args[3]);
                options
            }
        );

        //3
        assert_eq!(write_options(&[Nop, Nop, SelectiveAcknowledgement((args[0], args[1]), 
                                                                      [Some((args[2], args[3])), 
                                                                       Some((args[4], args[5])), 
                                                                       None]), 
                                   Nop]).options(), 
           &{
                let mut options = [
                    TCP_OPTION_ID_NOP, TCP_OPTION_ID_NOP, TCP_OPTION_ID_SELECTIVE_ACK, 26,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    TCP_OPTION_ID_NOP, TCP_OPTION_ID_END, 0, 0
                ];
                BigEndian::write_u32(&mut options[4..8], args[0]);
                BigEndian::write_u32(&mut options[8..12], args[1]);
                BigEndian::write_u32(&mut options[12..16], args[2]);
                BigEndian::write_u32(&mut options[16..20], args[3]);
                BigEndian::write_u32(&mut options[20..24], args[4]);
                BigEndian::write_u32(&mut options[24..28], args[5]);
                options
            }
        );

        //4
        assert_eq!(write_options(&[Nop, Nop, SelectiveAcknowledgement((args[0], args[1]), 
                                                                      [Some((args[2], args[3])), 
                                                                       Some((args[4], args[5])), 
                                                                       Some((args[6], args[7]))]), 
                                   Nop]).options(), 
           &{
                let mut options = [
                    TCP_OPTION_ID_NOP, TCP_OPTION_ID_NOP, TCP_OPTION_ID_SELECTIVE_ACK, 34,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    TCP_OPTION_ID_NOP, TCP_OPTION_ID_END, 0, 0
                ];
                BigEndian::write_u32(&mut options[4..8], args[0]);
                BigEndian::write_u32(&mut options[8..12], args[1]);
                BigEndian::write_u32(&mut options[12..16], args[2]);
                BigEndian::write_u32(&mut options[16..20], args[3]);
                BigEndian::write_u32(&mut options[20..24], args[4]);
                BigEndian::write_u32(&mut options[24..28], args[5]);
                BigEndian::write_u32(&mut options[28..32], args[6]);
                BigEndian::write_u32(&mut options[32..36], args[7]);
                options
            }[..]
        );
    }
}

proptest! {
    #[test]
    fn set_options_timestamp(arg0 in any::<u32>(),
                                        arg1 in any::<u32>()) {
        use TcpOptionElement::*;
        assert_eq!(write_options(&[Nop, Nop, Timestamp(arg0, arg1), Nop]).options(), 
           &{
                let mut options = [
                    TCP_OPTION_ID_NOP, TCP_OPTION_ID_NOP, TCP_OPTION_ID_TIMESTAMP, 10,
                    0, 0, 0, 0,
                    0, 0, 0, 0,
                    TCP_OPTION_ID_NOP, TCP_OPTION_ID_END, 0, 0
                ];
                BigEndian::write_u32(&mut options[4..8], arg0);
                BigEndian::write_u32(&mut options[8..12], arg1);
                options
            }
        );
    }
}

// TODO test too large

#[test]
fn set_options_not_enough_memory_error() {
    use TcpOptionElement::*;
    assert_eq!(Err(TcpOptionWriteError::NotEnoughSpace(41)),
               TcpHeader::default().set_options(
                    &[MaximumSegmentSize(1), //4
                      WindowScale(2), //+3 = 7
                      SelectiveAcknowledgementPermitted, //+2 = 9
                      SelectiveAcknowledgement((3,4), [Some((5,6)), None, None]), // + 18 = 27
                      Timestamp(5, 6), // + 10 = 37
                      Nop, Nop, Nop  // + 3 + 1 (for end)
                    ]));
    //test with all fields filled of the selective ack
    assert_eq!(Err(TcpOptionWriteError::NotEnoughSpace(41)),
           TcpHeader::default().set_options(
                &[Nop, // 1
                  SelectiveAcknowledgement((3,4), [Some((5,6)), Some((5,6)), Some((5,6))]), // + 34 = 35
                  MaximumSegmentSize(1), // + 4 = 39 
                  Nop // + 1 + 1 (for end) = 41
                ]));

    //test with all fields filled of the selective ack
    assert_eq!(Err(TcpOptionWriteError::NotEnoughSpace(41)),
           TcpHeader::default().set_options(
                &[Nop, // 1
                  SelectiveAcknowledgement((3,4), [None, None, None]), // + 10 = 11
                  Timestamp(1,2), // + 10 = 21
                  Timestamp(1,2), // + 10 = 31
                  MaximumSegmentSize(1), // + 4 = 35
                  Nop, Nop, Nop, Nop, Nop // + 5 + 1 (for end) = 41
                ]));
}


proptest! {
    #[test]
    fn read_write(ref input in tcp_any())
    {
        //serialize
        let mut buffer: Vec<u8> = Vec::with_capacity(60);
        input.write(&mut buffer).unwrap();
        //check length
        assert_eq!(input.data_offset() as usize * 4, buffer.len());
        //deserialize
        let result = TcpHeader::read(&mut Cursor::new(&buffer)).unwrap();
        //check equivalence
        assert_eq!(input, &result);
    }
}

proptest! {
    #[test]
    fn read_data_offset_too_small(ref input in tcp_any(),
                  data_offset in 0..TCP_MINIMUM_DATA_OFFSET)
    {
        //serialize
        let mut buffer: Vec<u8> = Vec::with_capacity(60);
        input.write(&mut buffer).unwrap();
        //insert the too small data offset into the raw stream
        buffer[12] = (buffer[12] & 0xf) | ((data_offset << 4) & 0xf0);
        //deserialize
        assert_matches!(TcpHeader::read(&mut Cursor::new(&buffer)),
                        Err(ReadError::TcpDataOffsetTooSmall(_)));
    }
}

proptest! {
    #[test]
    fn read_unexpected_eof(ref input in tcp_any())
    {
        //serialize
        let mut buffer: Vec<u8> = Vec::with_capacity(60);
        input.write(&mut buffer).unwrap();
        //deserialize
        let len = buffer.len() - 1;
        assert_matches!(TcpHeader::read(&mut Cursor::new(&buffer[..len])),
                        Err(ReadError::IoError(_)));
    }
}

proptest! {
    #[test]
    fn packet_slice_from_slice(ref input in tcp_any()) {
        //serialize
        let mut buffer: Vec<u8> = Vec::with_capacity(60);
        input.write(&mut buffer).unwrap();
        //create slice
        let slice = PacketSlice::<TcpHeader>::from_slice(&buffer).unwrap();
        //check all fields
        assert_eq!(input.source_port, slice.source_port());
        assert_eq!(input.destination_port, slice.destination_port());
        assert_eq!(input.sequence_number, slice.sequence_number());
        assert_eq!(input.acknowledgment_number, slice.acknowledgment_number());
        assert_eq!(input.data_offset(), slice.data_offset());
        assert_eq!(input.ns, slice.ns());
        assert_eq!(input.fin, slice.fin());
        assert_eq!(input.syn, slice.syn());
        assert_eq!(input.rst, slice.rst());
        assert_eq!(input.psh, slice.psh());
        assert_eq!(input.ack, slice.ack());
        assert_eq!(input.ece, slice.ece());
        assert_eq!(input.urg, slice.urg());
        assert_eq!(input.cwr, slice.cwr());
        assert_eq!(input.window_size, slice.window_size());
        assert_eq!(input.checksum, slice.checksum());
        assert_eq!(input.urgent_pointer, slice.urgent_pointer());
        assert_eq!(input.options(), slice.options());

        //check the to_header result
        assert_eq!(input, &slice.to_header());
    }
}

proptest! {
    #[test]
    fn packet_slice_from_slice_data_offset_too_small(ref input in tcp_any(),
                  data_offset in 0..TCP_MINIMUM_DATA_OFFSET)
    {
        //serialize
        let mut buffer: Vec<u8> = Vec::with_capacity(60);
        input.write(&mut buffer).unwrap();
        //insert the too small data offset into the raw stream
        buffer[12] = (buffer[12] & 0xf) | ((data_offset << 4) & 0xf0);
        //deserialize
        assert_matches!(PacketSlice::<TcpHeader>::from_slice(&buffer),
                        Err(ReadError::TcpDataOffsetTooSmall(_)));
    }
}

proptest! {
    #[test]
    fn debug_fmt(ref input in tcp_any())
    {
        assert_eq!(&format!("TcpHeader {{ source_port: {}, destination_port: {}, sequence_number: {}, acknowledgment_number: {}, data_offset: {}, ns: {}, fin: {}, syn: {}, rst: {}, psh: {}, ack: {}, urg: {}, ece: {}, cwr: {}, window_size: {}, checksum: {}, urgent_pointer: {} }}",
                input.source_port,
                input.destination_port,
                input.sequence_number,
                input.acknowledgment_number,
                input.data_offset(),
                input.ns,
                input.fin,
                input.syn,
                input.rst,
                input.psh,
                input.ack,
                input.urg,
                input.ece,
                input.cwr,
                input.window_size,
                input.checksum,
                input.urgent_pointer
            ),
            &format!("{:?}", input)
        );
    }
}
