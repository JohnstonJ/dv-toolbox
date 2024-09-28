use ffmpeg_next::format::sample;
use googletest::prelude::*;

use ffmpeg_next::codec;
use ffmpeg_next::codec::Compliance;
use ffmpeg_next::encoder;
use ffmpeg_next::frame;
use ffmpeg_next::{self as ffmpeg, ChannelLayout};
use insta::assert_snapshot;
use regex::Regex;
use std::{
    cell::RefCell,
    fs::File,
    io::{self, Cursor},
    rc::Rc,
};

use crate::{ffutil, testutil};

// ========== INPUT TESTS ==========

#[googletest::test]
fn test_open_seekable_input() {
    testutil::init_ffmpeg();

    let dv_format = ffutil::find_input_format("dv").unwrap();
    let path = testutil::test_resource("dv_multiframe/sony_good_quality.dv");
    let file = File::open(path).unwrap();
    let input_context =
        ffutil::open_seekable_input(Rc::new(RefCell::new(file)), Some(&dv_format)).unwrap();

    expect_that!(input_context.nb_streams(), eq(3));
    expect_that!(input_context.bit_rate(), eq(28771286));

    // test custom debug representation code paths
    let dbg_str = format!("{:#?}", input_context);
    let dbg_str = Regex::new(r"ptr: (0x[0-9a-f]*),").unwrap().replace(&dbg_str, "ptr: <redacted>,");
    assert_snapshot!(dbg_str, @r#"
    CustomFormatContextWrapper {
        format_context_type: "ffmpeg_next::format::context::input::Input",
        _io_context: IOContext {
            ptr: <redacted>,
            _opaque: Opaque {
                has_reader: true,
                has_writer: false,
                has_seeker: true,
            },
        },
    }
    "#);
}

struct FailingReader;

impl io::Read for FailingReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }
}

impl io::Seek for FailingReader {
    fn seek(&mut self, _pos: io::SeekFrom) -> io::Result<u64> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }
}

#[googletest::test]
fn test_open_seekable_input_failing_reader() {
    testutil::init_ffmpeg();

    let open_result = ffutil::open_seekable_input(Rc::new(RefCell::new(FailingReader)), None);

    expect_that!(open_result.err().unwrap(), eq(ffmpeg::Error::External));
}

struct InterruptingReader<R: io::Read + io::Seek> {
    wrapped: R,
    next_read_ok: bool,
    next_seek_ok: bool,
}

impl<R: io::Read + io::Seek> InterruptingReader<R> {
    fn new(wrapped: R) -> InterruptingReader<R> {
        return InterruptingReader { wrapped, next_read_ok: false, next_seek_ok: false };
    }
}

impl<R: io::Read + io::Seek> io::Read for InterruptingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let result = if self.next_read_ok {
            self.wrapped.read(buf)
        } else {
            Err(io::Error::from(io::ErrorKind::Interrupted))
        };
        self.next_read_ok = !self.next_read_ok;
        result
    }
}

impl<R: io::Read + io::Seek> io::Seek for InterruptingReader<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let result = if self.next_seek_ok {
            self.wrapped.seek(pos)
        } else {
            Err(io::Error::from(io::ErrorKind::Interrupted))
        };
        self.next_seek_ok = !self.next_seek_ok;
        result
    }
}

#[googletest::test]
fn test_open_seekable_input_interrupted() {
    testutil::init_ffmpeg();

    let dv_format = ffutil::find_input_format("dv").unwrap();
    let path = testutil::test_resource("dv_multiframe/sony_good_quality.dv");
    let file = InterruptingReader::new(File::open(path).unwrap());
    let input_context =
        ffutil::open_seekable_input(Rc::new(RefCell::new(file)), Some(&dv_format)).unwrap();

    expect_that!(input_context.nb_streams(), eq(3));
    expect_that!(input_context.bit_rate(), eq(28771286));
}

// ========== OUTPUT TESTS ==========

#[googletest::test]
fn test_open_seekable_output() {
    testutil::init_ffmpeg();

    // This test uses 8-bit unsigned PCM audio in raw format (i.e. no actual container).
    // This is really convenient because our raw audio "sample" byte stream is copied as-is
    // to the output I/O.
    let pcm_format = ffutil::guess_output_format(Some("u8"), None, None).unwrap();
    let pcm_encoder = ffmpeg::encoder::find_by_name("pcm_u8").unwrap();
    ffutil::format_supports_codec(&pcm_format, &pcm_encoder, Compliance::Normal).ok();

    // Open output container
    let output_buffer = Rc::new(RefCell::new(Cursor::new(Vec::<u8>::new())));
    let mut format_context =
        ffutil::open_seekable_output(output_buffer.clone(), &pcm_format).unwrap();

    // Set up codec context with initial parameters
    let codec_context = codec::context::Context::new_with_codec(pcm_encoder);
    let mut codec_context = codec_context.encoder().audio().unwrap();
    codec_context.set_rate(32000);
    codec_context.set_channel_layout(ChannelLayout::MONO);
    codec_context.set_time_base(ffmpeg::Rational(1, ffmpeg::ffi::AV_TIME_BASE));
    let codec_format = sample::Sample::U8(sample::Type::Packed);
    codec_context.set_format(codec_format);

    // Add a stream to the output
    let mut stream = format_context.add_stream_with(&codec_context).unwrap();
    stream.set_time_base(codec_context.time_base());
    let stream_index = stream.index();

    // Open the encoder
    let mut encoder = codec_context.open().unwrap();

    format_context.write_header().unwrap();

    // Create example audio frame, and then send it / get packets back
    const FRAME_BYTES: &str = "Hello World!";
    let mut frame =
        frame::Audio::new(codec_format, FRAME_BYTES.as_bytes().len(), ChannelLayout::MONO);
    frame.set_rate(32000);
    frame.set_pts(Some(0));
    frame.plane_mut(0).copy_from_slice(FRAME_BYTES.as_bytes());
    let mut proc_packets = |encoder: &mut encoder::Encoder, eof_error| loop {
        let mut packet = ffmpeg::Packet::empty();
        match encoder.receive_packet(&mut packet) {
            Ok(_) => {
                packet.set_stream(stream_index);
                packet.write_interleaved(&mut format_context).unwrap();
            }
            Err(err) if err == eof_error => break,
            Err(err) => panic!("Unexpected encoding error {}", err),
        }
    };
    encoder.send_frame(&frame).unwrap();
    proc_packets(&mut encoder, ffmpeg::Error::Other { errno: ffmpeg::error::EAGAIN });
    encoder.send_eof().unwrap();
    proc_packets(&mut encoder, ffmpeg::Error::Eof);

    // Send final trailer
    format_context.write_trailer().unwrap();

    // Now, we can assert on the final output...
    assert_that!(output_buffer.borrow().get_ref().as_slice(), eq(FRAME_BYTES.as_bytes()));

    // test custom debug representation code paths
    let dbg_str = format!("{:#?}", format_context);
    let dbg_str = Regex::new(r"ptr: (0x[0-9a-f]*),").unwrap().replace(&dbg_str, "ptr: <redacted>,");
    assert_snapshot!(dbg_str, @r#"
    CustomFormatContextWrapper {
        format_context_type: "ffmpeg_next::format::context::output::Output",
        _io_context: IOContext {
            ptr: <redacted>,
            _opaque: Opaque {
                has_reader: false,
                has_writer: true,
                has_seeker: true,
            },
        },
    }
    "#);
}
