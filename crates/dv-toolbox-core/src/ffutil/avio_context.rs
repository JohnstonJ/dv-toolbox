use std::{
    any::{self, TypeId},
    cell::RefCell,
    fmt,
    io::{self, SeekFrom},
    pin::Pin,
    ptr,
    rc::Rc,
    slice,
};

use crate::ioutil;
use derive_more::{Deref, DerefMut};
use ffmpeg_next as ffmpeg;
use ffmpeg_next::{format, format::context};

#[cfg(test)]
mod tests;

#[derive(Debug)]
struct IOContext {
    ptr: *mut ffmpeg::ffi::AVIOContext,
    _opaque: Pin<Box<Opaque>>,
}

struct Opaque {
    reader: Option<Rc<RefCell<dyn io::Read>>>,
    writer: Option<Rc<RefCell<dyn io::Write>>>,
    seeker: Option<Rc<RefCell<dyn io::Seek>>>,
}

impl fmt::Debug for Opaque {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Opaque")
            .field("has_reader", &self.reader.is_some())
            .field("has_writer", &self.writer.is_some())
            .field("has_seeker", &self.seeker.is_some())
            .finish()
    }
}

#[derive(Debug)]
enum IOContextType {
    Reader,
    #[allow(dead_code)]
    Writer,
}

extern "C" fn read_packet(
    opaque: *mut libc::c_void,
    buf: *mut u8,
    buf_size: libc::c_int,
) -> libc::c_int {
    if buf_size == 0 {
        return 0;
    }
    let buf_usize = usize::try_from(buf_size).unwrap();
    let buf = unsafe { slice::from_raw_parts_mut(buf, buf_usize) };
    let opaque = unsafe { &mut *(opaque as *mut Opaque) };
    let mut reader = (*opaque.reader.as_mut().unwrap()).try_borrow_mut().unwrap();
    // NOTE: returning less bytes than requested is OK:  https://stackoverflow.com/a/75050571
    // You can see a suitable loop to handle that in aviobuf.c: avio_read function
    let bytes_read = match ioutil::retry_if_interrupted(|| reader.read(buf)) {
        // FFmpeg doesn't allow zero bytes read as a valid return value
        Ok(0) => return ffmpeg::ffi::AVERROR_EOF,
        Ok(n) => n,
        Err(_) => return ffmpeg::ffi::AVERROR_EXTERNAL,
    };
    assert!(bytes_read <= buf_usize); // avoid undefined behavior in FFmpeg
    libc::c_int::try_from(bytes_read).unwrap()
}

extern "C" fn write_packet(
    opaque: *mut libc::c_void,
    buf: *const u8,
    buf_size: libc::c_int,
) -> libc::c_int {
    let buf_usize = usize::try_from(buf_size).unwrap();
    let buf = unsafe { slice::from_raw_parts(buf, buf_usize) };
    let opaque = unsafe { &mut *(opaque as *mut Opaque) };
    let mut writer = (*opaque.writer.as_mut().unwrap()).try_borrow_mut().unwrap();
    // NOTE: Examining aviobuf.c: avio_write and flush_buffer functions indicates that we MUST
    // write ALL the buf_size bytes that were requested.  They won't come back to finish the job
    // if we do a partial write.
    if writer.write_all(buf).is_err() {
        return ffmpeg::ffi::AVERROR_EXTERNAL;
    }
    buf_size
}

extern "C" fn seek_fn(opaque: *mut libc::c_void, offset: i64, whence: libc::c_int) -> i64 {
    let opaque = unsafe { &mut *(opaque as *mut Opaque) };
    let mut seeker = (*opaque.seeker.as_mut().unwrap()).try_borrow_mut().unwrap();
    let whence = whence & !ffmpeg::ffi::AVSEEK_FORCE;
    let result = match whence {
        ffmpeg::ffi::SEEK_SET => ioutil::retry_if_interrupted(|| {
            seeker.seek(SeekFrom::Start(u64::try_from(offset).unwrap()))
        }),
        ffmpeg::ffi::SEEK_CUR => {
            ioutil::retry_if_interrupted(|| seeker.seek(SeekFrom::Current(offset)))
        }
        ffmpeg::ffi::SEEK_END => {
            ioutil::retry_if_interrupted(|| seeker.seek(SeekFrom::End(offset)))
        }
        // Someday, Rust will provide a better way:
        // https://doc.rust-lang.org/std/io/trait.Seek.html#method.stream_len
        //
        // Note that the comments for AVSEEK_SIZE say "If it is not supported then the seek
        // function will return <0."  But I'm not sure if I trust that: the avio_seek function
        // doesn't provide any sort of fallback otherwise.  Safest thing is probably to implement
        // this.  The comment here provides further hinting that we may need to implement:
        // https://stackoverflow.com/questions/43621303/ffmpeg-avformat-open-input-with-custom-stream-object#comment74327001_43626066
        ffmpeg::ffi::AVSEEK_SIZE => (|| {
            let old_pos = ioutil::retry_if_interrupted(|| seeker.stream_position())?;
            let len = ioutil::retry_if_interrupted(|| seeker.seek(SeekFrom::End(0)))?;
            if old_pos != len {
                ioutil::retry_if_interrupted(|| seeker.seek(SeekFrom::Start(old_pos)))?;
            }
            Ok(len)
        })(),
        _ => return ffmpeg::ffi::AVERROR(ffmpeg::ffi::EINVAL).into(),
    };
    match result {
        Ok(pos) => i64::try_from(pos).unwrap(),
        Err(_) => ffmpeg::ffi::AVERROR_EXTERNAL.into(),
    }
}

impl IOContext {
    fn new(
        ctx_type: IOContextType,
        reader: Option<Rc<RefCell<dyn io::Read>>>,
        writer: Option<Rc<RefCell<dyn io::Write>>>,
        seeker: Option<Rc<RefCell<dyn io::Seek>>>,
    ) -> IOContext {
        let buffer_size = page_size::get();
        let buffer = unsafe { ffmpeg::ffi::av_malloc(buffer_size) };
        assert!(!buffer.is_null());
        let mut opaque = Box::pin(Opaque { reader, writer, seeker });
        let ctx = unsafe {
            ffmpeg::ffi::avio_alloc_context(
                buffer as *mut libc::c_uchar,
                libc::c_int::try_from(buffer_size).unwrap(),
                match ctx_type {
                    IOContextType::Reader => 0,
                    IOContextType::Writer => 1,
                },
                ptr::addr_of_mut!(*opaque.as_mut().get_unchecked_mut()) as *mut libc::c_void,
                opaque.reader.as_deref().map(|_| read_packet as _),
                opaque.writer.as_deref().map(|_| write_packet as _),
                opaque.seeker.as_deref().map(|_| seek_fn as _),
            )
        };
        // in practice, a null ctx means malloc failed
        if ctx.is_null() {
            unsafe { ffmpeg::ffi::av_freep(buffer) };
        }
        assert!(!ctx.is_null());

        IOContext { ptr: ctx, _opaque: opaque }
    }
}

impl Drop for IOContext {
    fn drop(&mut self) {
        let ctx = unsafe { self.ptr.as_mut() }.unwrap();
        if !ctx.buffer.is_null() {
            let buf_ptr = ptr::addr_of_mut!(ctx.buffer);
            unsafe { ffmpeg::ffi::av_freep(buf_ptr as *mut libc::c_void) };
        }
        unsafe { ffmpeg::ffi::avio_context_free(ptr::addr_of_mut!(self.ptr)) };
    }
}

unsafe impl Send for IOContext {}

/// Wraps the format context along with the I/O context, so that they are both correctly freed in
/// the right order.
#[derive(Deref, DerefMut)]
pub struct CustomFormatContextWrapper<C: 'static> {
    // Order is critical here to avoid undefined behavior: we need to drop format_context
    // before io_context!
    #[deref]
    #[deref_mut]
    format_context: C,
    _io_context: IOContext,

    context_ptr: *mut ffmpeg::ffi::AVFormatContext,
}

impl<C: 'static> Drop for CustomFormatContextWrapper<C> {
    fn drop(&mut self) {
        // Output files are a special case because ffmpeg_next tries to call avio_close on the
        // pointer to our custom I/O context, which is obvious undefined behavior.  We need to
        // disconnect our I/O context from the structure, and then it will work because avio_close
        // will safely ignore NULL pointers.
        //
        // For input files, FFmpeg doesn't do anything special, and FFmpeg advertises that they
        // won't themselves close user-provided I/O contexts.
        if TypeId::of::<C>() == TypeId::of::<context::Output>() {
            unsafe { self.context_ptr.as_mut() }.unwrap().pb = ptr::null_mut();
        }
    }
}

// FFmpeg context types unfortunately don't implement fmt::Debug
impl<C> fmt::Debug for CustomFormatContextWrapper<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomFormatContextWrapper")
            .field("format_context_type", &any::type_name::<C>())
            .field("_io_context", &self._io_context)
            .finish()
    }
}

/// Opens a media container format for input in FFmpeg.
///
/// Any source can be used, so long as it implements the standard [`io::Read`] and [`io::Seek`]
/// traits.  If the [`format::Input`] is not provided, then FFmpeg will probe and try to guess it.
/// It's best to provide this if known.
pub fn open_seekable_input<R: io::Read + io::Seek + 'static>(
    seekable_reader: Rc<RefCell<R>>,
    format: Option<&format::Input>,
) -> Result<CustomFormatContextWrapper<context::Input>, ffmpeg::Error> {
    // Create IOContext for using the generic I/O traits
    let io_ctx: IOContext = IOContext::new(
        IOContextType::Reader,
        Some(seekable_reader.clone()),
        None,
        Some(seekable_reader.clone()),
    );

    // Set up input format context
    let mut ctx = unsafe { ffmpeg::ffi::avformat_alloc_context() };
    let ctx_mut = unsafe { ctx.as_mut() }.unwrap();
    ctx_mut.pb = io_ctx.ptr;

    // Open the input
    let err = unsafe {
        ffmpeg::ffi::avformat_open_input(
            &mut ctx,
            ptr::null_mut(),
            format.map_or(ptr::null(), |f| f.as_ptr()),
            ptr::null_mut(),
        )
    };
    if err < 0 {
        // Note that avformat_open_input will free the context if there is an error
        return Err(ffmpeg::Error::from(err));
    }

    // Now, rely on Input to free the memory
    let mut ctx = unsafe { context::Input::wrap(ctx) };

    // Probe to find stream information
    let err = unsafe { ffmpeg::ffi::avformat_find_stream_info(ctx.as_mut_ptr(), ptr::null_mut()) };
    if err < 0 {
        return Err(ffmpeg::Error::from(err));
    }

    Ok(CustomFormatContextWrapper {
        format_context: ctx,
        _io_context: io_ctx,
        context_ptr: ctx_mut,
    })
}

/// Opens a media container format for output in FFmpeg.
///
/// Any output can be used, so long as it implements the standard [`io::Write`] and [`io::Seek`]
/// traits.
pub fn open_seekable_output<R: io::Write + io::Seek + 'static>(
    seekable_writer: Rc<RefCell<R>>,
    format: &format::Output,
) -> Result<CustomFormatContextWrapper<context::Output>, ffmpeg::Error> {
    // Create IOContext for using the generic I/O traits
    let io_ctx: IOContext = IOContext::new(
        IOContextType::Writer,
        None,
        Some(seekable_writer.clone()),
        Some(seekable_writer.clone()),
    );

    // Allocate output format context
    let mut ctx: *mut ffmpeg::ffi::AVFormatContext = ptr::null_mut();
    let err = unsafe {
        ffmpeg::ffi::avformat_alloc_output_context2(
            &mut ctx,
            format.as_ptr(),
            ptr::null(),
            ptr::null(),
        )
    };
    if err < 0 {
        return Err(ffmpeg::Error::from(err));
    }
    assert!(!ctx.is_null());

    // Now, rely on Output to free the memory
    let mut ctx = unsafe { context::Output::wrap(ctx) };

    // Assign IOContext to the output format context
    let ctx_mut = unsafe { ctx.as_mut_ptr().as_mut() }.unwrap();
    ctx_mut.pb = io_ctx.ptr;

    Ok(CustomFormatContextWrapper {
        format_context: ctx,
        _io_context: io_ctx,
        context_ptr: ctx_mut,
    })
}
