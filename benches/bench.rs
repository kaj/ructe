#![feature(test)]
extern crate test;
use ructe::templates::{Html, ToHtml};
use std::io::Write;
use test::bench::{Bencher, black_box};

#[bench]
fn raw(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(500);
    b.iter(|| {
        buf.clear();
        black_box(Html("Hello <World>").to_html(&mut buf)).unwrap();
    });
}
#[bench]
fn raw_baseline(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(500);
    b.iter(|| {
        buf.clear();
        black_box(write!(&mut buf, "{}", "Hello <World>")).unwrap();
    });
}

#[bench]
fn escaped_no_op(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(500);
    b.iter(|| {
        buf.clear();
        black_box("Hello World!!".to_html(&mut buf)).unwrap();
    });
}

/// Note: Numbers can't contain anything that should be escaped, so
/// with proper specialization, this should be as fast as
/// escaped_nums_h and escaped_nums_baseline.
#[bench]
fn escaped_nums(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(500);
    b.iter(|| {
        buf.clear();
        black_box(12_345.to_html(&mut buf)).unwrap();
    });
}

/// Turn of escapeing for the number by claiming it to be pre-escaped.
#[bench]
fn escaped_nums_h(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(500);
    b.iter(|| {
        buf.clear();
        black_box(Html(12_345).to_html(&mut buf)).unwrap();
    });
}

/// Raw rust write of the same number for comparision.
#[bench]
fn escaped_nums_baseline(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(500);
    b.iter(|| {
        buf.clear();
        black_box(write!(&mut buf, "{}", 12_345)).unwrap();
    });
}

#[bench]
fn escaped_short_impl(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(500);
    b.iter(|| {
        buf.clear();
        black_box("Hello <World>".to_html(&mut buf)).unwrap();
    });
}

#[bench]
fn escaped_short_dyn(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(500);
    b.iter(|| {
        buf.clear();
        let mut dynwrite: &mut dyn Write = black_box(&mut buf);
        black_box("Hello <World>".to_html(&mut dynwrite)).unwrap();
    });
}

#[bench]
fn escaped_long(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(1000);
    let text = "Lorem ipsum dolor sit amet, consectetur adipisicing elit, \
                sed do eiusmod tempor incididunt ut labore et dolore magna \
                aliqua. Ut enim ad minim veniam, quis nostrud exercitation \
                ullamco laboris nisi ut aliquip ex ea commodo consequat.\n \
                Duis aute irure dolor in reprehenderit <in> voluptate velit \
                esse cillum dolore eu fugiat nulla pariatur. Excepteur sint \
                occaecat cupidatat non proident, sunt in culpa qui officia \
                deserunt mollit anim id est laborum.\n";
    b.iter(|| {
        buf.clear();
        black_box(text.to_html(&mut buf)).unwrap();
    });
}
