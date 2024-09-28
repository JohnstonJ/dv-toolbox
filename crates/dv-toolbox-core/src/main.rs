//! TODO

use std::{cell::RefCell, fs::File, rc::Rc};

use ffmpeg_next as ffmpeg;

fn main() {
    ffmpeg::init().unwrap();

    let dv_format = dv_toolbox_core::ffutil::find_input_format("dv").unwrap();
    let file = File::open("C:\\scrap\\LongPlay.dv").unwrap();
    let input =
        dv_toolbox_core::ffutil::open_seekable_input(Rc::new(RefCell::new(file)), Some(&dv_format))
            .expect("opening failed");
    println!("{}", input.nb_streams());
    println!("{}", input.bit_rate());
}
