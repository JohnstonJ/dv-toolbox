use std::{
    cell::RefCell,
    fs::File,
    io::{self, Cursor},
    rc::Rc,
};

use googletest::prelude::*;
use insta::assert_snapshot;
use regex::Regex;
use rsmpeg::{
    avcodec::{AVCodec, AVCodecContext},
    avformat::{AVInputFormat, AVOutputFormat},
    avutil::{self, AVFrame},
    error::RsmpegError,
};

use crate::{ffutil, ffutil::Compliance, testutil};

// ========== INPUT TESTS ==========

#[googletest::test]
fn test_open_seekable_input() {
    let dv_format = AVInputFormat::find(c"dv").unwrap();
    let path = testutil::test_resource("dv_multiframe/sony_good_quality.dv");
    let file = File::open(path).unwrap();
    let input_context =
        ffutil::open_seekable_input(Rc::new(RefCell::new(file)), Some(&dv_format)).unwrap();

    expect_that!(input_context.nb_streams, eq(3));
    expect_that!(input_context.bit_rate, eq(28771286));

    // test custom debug representation code paths
    let dbg_str = format!("{:#?}", input_context);
    let dbg_str = Regex::new(r"ptr: (0x[0-9a-f]*),").unwrap().replace(&dbg_str, "ptr: <redacted>,");
    assert_snapshot!(dbg_str, @r#"
    CustomFormatContextWrapper {
        format_context_type: "rsmpeg::avformat::avformat::AVFormatContextInput",
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
    let open_result = ffutil::open_seekable_input(Rc::new(RefCell::new(FailingReader)), None);

    expect_that!(open_result, err(eq(&RsmpegError::OpenInputError(rsmpeg::ffi::AVERROR_EXTERNAL))));
}

struct InterruptingReader<R: io::Read + io::Seek> {
    wrapped: R,
    next_read_ok: bool,
    next_seek_ok: bool,
}

impl<R: io::Read + io::Seek> InterruptingReader<R> {
    fn new(wrapped: R) -> InterruptingReader<R> {
        InterruptingReader { wrapped, next_read_ok: false, next_seek_ok: false }
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
    let dv_format = AVInputFormat::find(c"dv").unwrap();
    let path = testutil::test_resource("dv_multiframe/sony_good_quality.dv");
    let file = InterruptingReader::new(File::open(path).unwrap());
    let input_context =
        ffutil::open_seekable_input(Rc::new(RefCell::new(file)), Some(&dv_format)).unwrap();

    expect_that!(input_context.nb_streams, eq(3));
    expect_that!(input_context.bit_rate, eq(28771286));
}

// ========== OUTPUT TESTS ==========

#[googletest::test]
fn test_open_seekable_output() {
    // This test uses 8-bit unsigned PCM audio in raw format (i.e. no actual container).
    // This is really convenient because our raw audio "sample" byte stream is copied as-is
    // to the output I/O.
    let pcm_format = AVOutputFormat::guess_format(Some(c"u8"), None, None).unwrap();
    let pcm_encoder = AVCodec::find_encoder_by_name(c"pcm_u8").unwrap();
    ffutil::format_supports_codec(&pcm_format, &pcm_encoder, Compliance::Normal).ok();

    // Open output container
    let output_buffer = Rc::new(RefCell::new(Cursor::new(Vec::<u8>::new())));
    let mut format_context =
        ffutil::open_seekable_output(output_buffer.clone(), &pcm_format).unwrap();

    // Set up codec context with initial parameters
    let mut codec_context = AVCodecContext::new(&pcm_encoder);
    codec_context.set_sample_rate(32000);
    codec_context
        .set_ch_layout(avutil::AVChannelLayout::from_string(c"mono").unwrap().into_inner());
    codec_context.set_time_base(rsmpeg::ffi::AVRational {
        num: 1,
        den: rsmpeg::ffi::AV_TIME_BASE.try_into().unwrap(),
    });
    codec_context.set_sample_fmt(rsmpeg::ffi::AV_SAMPLE_FMT_U8);

    // Add a stream to the output
    let mut stream = format_context.new_stream();
    stream.codecpar_mut().from_context(&codec_context);
    stream.set_time_base(codec_context.time_base);
    let stream_index = stream.index;
    std::mem::drop(stream); // stream is mutably borrowing from format_context

    // Open the encoder
    codec_context.open(None).unwrap();

    format_context.write_header(&mut None).unwrap();

    // Create example audio frame, and then send it / get packets back
    const FRAME_BYTES: &str = "Hello World!";
    let mut frame = AVFrame::new();
    frame.set_format(rsmpeg::ffi::AV_SAMPLE_FMT_U8);
    frame.set_nb_samples(FRAME_BYTES.as_bytes().len().try_into().unwrap());
    frame.set_ch_layout(avutil::AVChannelLayout::from_string(c"mono").unwrap().into_inner());
    frame.set_sample_rate(32000);
    frame.set_pts(0);
    frame.alloc_buffer().unwrap();
    unsafe { std::slice::from_raw_parts_mut(frame.data_mut()[0], FRAME_BYTES.as_bytes().len()) }
        .copy_from_slice(FRAME_BYTES.as_bytes());
    let mut proc_packets = |codec_context: &mut AVCodecContext, eof_error| loop {
        let mut received = codec_context.receive_packet();
        match received {
            Ok(ref mut packet) => {
                packet.set_stream_index(stream_index);
                format_context.interleaved_write_frame(packet).unwrap();
            }
            Err(err) if err == eof_error => break,
            Err(err) => panic!("Unexpected encoding error {}", err),
        }
    };
    codec_context.send_frame(Some(&frame)).unwrap();
    proc_packets(&mut codec_context, RsmpegError::EncoderDrainError);
    codec_context.send_frame(None).unwrap();
    proc_packets(&mut codec_context, RsmpegError::EncoderFlushedError);

    // Send final trailer
    format_context.write_trailer().unwrap();

    // Now, we can assert on the final output...
    expect_that!(output_buffer.borrow().get_ref().as_slice(), eq(FRAME_BYTES.as_bytes()));

    // test custom debug representation code paths
    let dbg_str = format!("{:#?}", format_context);
    let dbg_str = Regex::new(r"ptr: (0x[0-9a-f]*),").unwrap().replace(&dbg_str, "ptr: <redacted>,");
    assert_snapshot!(dbg_str, @r#"
    CustomFormatContextWrapper {
        format_context_type: "rsmpeg::avformat::avformat::AVFormatContextOutput",
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
