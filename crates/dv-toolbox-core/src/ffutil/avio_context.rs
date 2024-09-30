use std::{
    any::{self, TypeId},
    cell::RefCell,
    fmt,
    io::{self, SeekFrom},
    os::raw,
    pin::Pin,
    ptr,
    rc::Rc,
    slice,
};

use derive_more::{Deref, DerefMut};
use rsmpeg::{
    avformat::{AVFormatContextInput, AVFormatContextOutput, AVInputFormat, AVOutputFormat},
    avutil::AVMem,
    error::RsmpegError,
    UnsafeDerefMut,
};

use crate::ioutil;

#[cfg(test)]
mod tests;

#[derive(Debug)]
struct IOContext {
    ptr: ptr::NonNull<rsmpeg::ffi::AVIOContext>,
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
    Writer,
}

extern "C" fn read_packet(
    opaque: *mut raw::c_void,
    buf: *mut u8,
    buf_size: raw::c_int,
) -> raw::c_int {
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
        Ok(0) => return rsmpeg::ffi::AVERROR_EOF,
        Ok(n) => n,
        Err(_) => return rsmpeg::ffi::AVERROR_EXTERNAL,
    };
    assert!(bytes_read <= buf_usize); // avoid undefined behavior in FFmpeg
    raw::c_int::try_from(bytes_read).unwrap()
}

extern "C" fn write_packet(
    opaque: *mut raw::c_void,
    buf: *const u8,
    buf_size: raw::c_int,
) -> raw::c_int {
    let buf_usize = usize::try_from(buf_size).unwrap();
    let buf = unsafe { slice::from_raw_parts(buf, buf_usize) };
    let opaque = unsafe { &mut *(opaque as *mut Opaque) };
    let mut writer = (*opaque.writer.as_mut().unwrap()).try_borrow_mut().unwrap();
    // NOTE: Examining aviobuf.c: avio_write and flush_buffer functions indicates that we MUST
    // write ALL the buf_size bytes that were requested.  They won't come back to finish the job
    // if we do a partial write.
    if writer.write_all(buf).is_err() {
        return rsmpeg::ffi::AVERROR_EXTERNAL;
    }
    buf_size
}

extern "C" fn seek_fn(opaque: *mut raw::c_void, offset: i64, whence: raw::c_int) -> i64 {
    let opaque = unsafe { &mut *(opaque as *mut Opaque) };
    let mut seeker = (*opaque.seeker.as_mut().unwrap()).try_borrow_mut().unwrap();
    let whence = whence & !raw::c_int::try_from(rsmpeg::ffi::AVSEEK_FORCE).unwrap();
    let result = match whence {
        w if w == rsmpeg::ffi::SEEK_SET.try_into().unwrap() => ioutil::retry_if_interrupted(|| {
            seeker.seek(SeekFrom::Start(u64::try_from(offset).unwrap()))
        }),
        w if w == rsmpeg::ffi::SEEK_CUR.try_into().unwrap() => {
            ioutil::retry_if_interrupted(|| seeker.seek(SeekFrom::Current(offset)))
        }
        w if w == rsmpeg::ffi::SEEK_END.try_into().unwrap() => {
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
        w if w == rsmpeg::ffi::AVSEEK_SIZE.try_into().unwrap() => (|| {
            let old_pos = ioutil::retry_if_interrupted(|| seeker.stream_position())?;
            let len = ioutil::retry_if_interrupted(|| seeker.seek(SeekFrom::End(0)))?;
            if old_pos != len {
                ioutil::retry_if_interrupted(|| seeker.seek(SeekFrom::Start(old_pos)))?;
            }
            Ok(len)
        })(),
        _ => return rsmpeg::ffi::AVERROR(rsmpeg::ffi::EINVAL).into(),
    };
    match result {
        Ok(pos) => i64::try_from(pos).unwrap(),
        Err(_) => rsmpeg::ffi::AVERROR_EXTERNAL.into(),
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
        let buffer = ptr::NonNull::new(unsafe { rsmpeg::ffi::av_malloc(buffer_size) }).unwrap();
        let mut opaque = Box::pin(Opaque { reader, writer, seeker });
        let ctx = unsafe {
            rsmpeg::ffi::avio_alloc_context(
                buffer.as_ptr() as *mut raw::c_uchar,
                raw::c_int::try_from(buffer_size).unwrap(),
                match ctx_type {
                    IOContextType::Reader => 0,
                    IOContextType::Writer => 1,
                },
                ptr::addr_of_mut!(*opaque.as_mut().get_unchecked_mut()) as *mut raw::c_void,
                opaque.reader.as_deref().map(|_| read_packet as _),
                opaque.writer.as_deref().map(|_| write_packet as _),
                opaque.seeker.as_deref().map(|_| seek_fn as _),
            )
        };
        // in practice, a null ctx means malloc failed
        if ctx.is_null() {
            unsafe { rsmpeg::ffi::av_freep(buffer.as_ptr()) };
        }

        IOContext { ptr: ptr::NonNull::new(ctx).unwrap(), _opaque: opaque }
    }
}

impl Drop for IOContext {
    fn drop(&mut self) {
        if let Some(buffer) = ptr::NonNull::new(unsafe { self.ptr.as_ref() }.buffer) {
            std::mem::drop(unsafe { AVMem::from_raw(buffer) });
            unsafe { self.ptr.as_mut() }.buffer = ptr::null_mut();
        }

        let mut self_ptr = self.ptr.as_ptr();
        unsafe { rsmpeg::ffi::avio_context_free(ptr::addr_of_mut!(self_ptr)) };
    }
}

unsafe impl Send for IOContext {}

/// Wraps the format context along with the I/O context, so that they are both correctly freed in
/// the right order.
#[derive(Deref, DerefMut)]
pub(crate) struct CustomFormatContextWrapper<C>
where
    C: Deref<Target = rsmpeg::ffi::AVFormatContext>
        + UnsafeDerefMut<Target = rsmpeg::ffi::AVFormatContext>
        + 'static,
{
    // Order is critical here to avoid undefined behavior: we need to drop format_context
    // before io_context!
    #[deref]
    #[deref_mut]
    format_context: C,
    _io_context: IOContext,
}

impl<C> Drop for CustomFormatContextWrapper<C>
where
    C: Deref<Target = rsmpeg::ffi::AVFormatContext>
        + UnsafeDerefMut<Target = rsmpeg::ffi::AVFormatContext>
        + 'static,
{
    fn drop(&mut self) {
        // Output files are a special case because rsmpeg tries to free the pointer to our custom
        // I/O context.  To minimize the risk of undefined behavior, we need to disconnect our
        // I/O context from the structure so that no attempt is made to free it.  We will later free
        // it ourselves when the IOContext itself is dropped.
        //
        // For input files, rsmpeg doesn't do anything special, and FFmpeg advertises that they
        // won't themselves close user-provided I/O contexts.
        if TypeId::of::<C>() == TypeId::of::<AVFormatContextOutput>() {
            unsafe { self.format_context.deref_mut() }.pb = ptr::null_mut();
        }
    }
}

// FFmpeg context types unfortunately don't implement fmt::Debug
impl<C> fmt::Debug for CustomFormatContextWrapper<C>
where
    C: Deref<Target = rsmpeg::ffi::AVFormatContext>
        + UnsafeDerefMut<Target = rsmpeg::ffi::AVFormatContext>
        + 'static,
{
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
/// traits.  If the [`AVInputFormat`] is not provided, then FFmpeg will probe and try to guess it.
/// It's best to provide this if known.
pub(crate) fn open_seekable_input<R: io::Read + io::Seek + 'static>(
    seekable_reader: Rc<RefCell<R>>,
    format: Option<&AVInputFormat>,
) -> rsmpeg::error::Result<CustomFormatContextWrapper<AVFormatContextInput>> {
    // Create IOContext for using the generic I/O traits
    let io_ctx: IOContext = IOContext::new(
        IOContextType::Reader,
        Some(seekable_reader.clone()),
        None,
        Some(seekable_reader.clone()),
    );

    // Set up input format context
    let mut ctx = unsafe { rsmpeg::ffi::avformat_alloc_context() };
    let ctx_mut = unsafe { ctx.as_mut() }.unwrap();
    ctx_mut.pb = io_ctx.ptr.as_ptr();

    // Open the input
    let err = unsafe {
        rsmpeg::ffi::avformat_open_input(
            &mut ctx,
            ptr::null_mut(),
            format.map_or(ptr::null(), |f| f.as_ptr()),
            ptr::null_mut(),
        )
    };
    if err < 0 {
        // Note that avformat_open_input will free the context if there is an error
        return Err(RsmpegError::OpenInputError(err));
    }

    // Now, rely on AVFormatContextInput to free the memory
    let mut ctx = unsafe { AVFormatContextInput::from_raw(ptr::NonNull::new(ctx).unwrap()) };

    // Probe to find stream information
    let err = unsafe { rsmpeg::ffi::avformat_find_stream_info(ctx.as_mut_ptr(), ptr::null_mut()) };
    if err < 0 {
        return Err(RsmpegError::FindStreamInfoError(err));
    }

    Ok(CustomFormatContextWrapper { format_context: ctx, _io_context: io_ctx })
}

/// Opens a media container format for output in FFmpeg.
///
/// Any output can be used, so long as it implements the standard [`io::Write`] and [`io::Seek`]
/// traits.
pub(crate) fn open_seekable_output<R: io::Write + io::Seek + 'static>(
    seekable_writer: Rc<RefCell<R>>,
    format: &AVOutputFormat,
) -> rsmpeg::error::Result<CustomFormatContextWrapper<AVFormatContextOutput>> {
    // Create IOContext for using the generic I/O traits
    let io_ctx: IOContext = IOContext::new(
        IOContextType::Writer,
        None,
        Some(seekable_writer.clone()),
        Some(seekable_writer.clone()),
    );

    // Allocate output format context
    let mut ctx: *mut rsmpeg::ffi::AVFormatContext = ptr::null_mut();
    let err = unsafe {
        rsmpeg::ffi::avformat_alloc_output_context2(
            &mut ctx,
            format.as_ptr(),
            ptr::null(),
            ptr::null(),
        )
    };
    if err < 0 {
        return Err(err.into());
    }

    // Now, rely on AVFormatContextOutput to free the memory
    let mut ctx = unsafe { AVFormatContextOutput::from_raw(ptr::NonNull::new(ctx).unwrap()) };

    // Assign IOContext to the output format context
    let n = io_ctx.ptr.as_ptr();
    unsafe { ctx.deref_mut() }.pb = n;

    Ok(CustomFormatContextWrapper { format_context: ctx, _io_context: io_ctx })
}
