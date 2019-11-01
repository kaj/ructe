#![feature(test)]
extern crate ructe;
extern crate test;
use std::fmt::Display;
use std::io;
use std::io::Write;
use test::Bencher;

include!("../src/template_utils.rs");

#[bench]
fn raw(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(10000);
    b.iter(|| {
        buf.clear();
        raw_inner(&mut buf);
        buf.len() // prevents optimizing writes out
    });
}

/// real template is a function that takes a generic `Write` parameter, so non-inlineable
/// function should simulate that instead of allowing optimizer to specialize for `Vec`
#[inline(never)]
fn raw_inner(buf: &mut impl Write) {
    // inner loop to stress escaping more than buffer allocation
    for _ in 0..1000 {
        let h = Html("Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod");
        h.to_html(buf).unwrap();
    }
}

#[bench]
fn escaped_no_op(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(10000);
    b.iter(|| {
        buf.clear();
        escaped_no_op_inner(&mut buf);
        buf.len()
    });
}

#[inline(never)]
fn escaped_no_op_inner(buf: &mut impl Write) {
    let h = "hello world";
    for _ in 0..1000 {
        h.to_html(buf).unwrap();
        h.to_html(buf).unwrap();
        h.to_html(buf).unwrap();
    }
}

#[bench]
fn escaped_nums(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(10000);
    b.iter(|| {
        buf.clear();
        escaped_nums_inner(&mut buf);
        buf.len()
    });
}

#[inline(never)]
fn escaped_nums_inner(buf: &mut impl Write) {
    for i in 0..1000 {
        i.to_html(buf).unwrap();
        5.to_html(buf).unwrap();
        i.to_html(buf).unwrap();
    }
}

#[bench]
fn escaped_short_impl(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(10000);
    b.iter(|| {
        buf.clear();
        escaped_short_inner(&mut buf);
        buf.len()
    });
}

#[bench]
fn escaped_short_dyn(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(10000);
    b.iter(|| {
        buf.clear();
        let mut dynwrite: &mut dyn Write = &mut buf;
        escaped_short_inner(&mut dynwrite);
        buf.len()
    });
}

#[inline(never)]
fn escaped_short_inner(buf: &mut impl Write) {
    for _ in 0..1000 {
        "hello&world".to_html(buf).unwrap();
        "hi".to_html(buf).unwrap();
        "hello=world!".to_html(buf).unwrap();
    }
}

#[bench]
fn escaped_long(b: &mut Bencher) {
    let mut buf = Vec::with_capacity(10000);
    b.iter(|| {
        buf.clear();
        escaped_long_inner(&mut buf);
        buf.len()
    });
}

#[inline(never)]
fn escaped_long_inner(buf: &mut impl Write) {
    for _ in 0..100 {
        let h = "Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod
tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam,
quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo
consequat. Duis aute irure dolor in reprehenderit <in> voluptate velit esse
cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non
proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        h.to_html(buf).unwrap();
        h.to_html(buf).unwrap();
        h.to_html(buf).unwrap();
    }
}
