use super::*;
use crate::testutil;
use googletest::prelude::*;
use rstest::rstest;
use std::fs::File;

struct InfoReadTestCase<'a> {
    filename: &'a str,
    info: Info,
}

const INFO_READ_SONY_GOOD_QUALITY: InfoReadTestCase = InfoReadTestCase {
    filename: "dv_multiframe/sony_good_quality.dv",
    info: Info { file_size: 600_000 },
};

#[googletest::test]
#[rstest]
#[case::sony_good_quality(INFO_READ_SONY_GOOD_QUALITY)]
fn test_info_read(#[case] tc: InfoReadTestCase) {
    testutil::init_ffmpeg();

    let file = File::open(testutil::test_resource(tc.filename)).unwrap();
    let info = Info::read(Rc::new(RefCell::new(file))).unwrap();

    expect_that!(info, eq(tc.info));
}
