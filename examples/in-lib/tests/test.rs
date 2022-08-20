#[test]
fn use_template() {
    let mut buf = Vec::new();
    in_lib::hello_args_html(&mut buf, "World").unwrap();
    assert_eq!(
        String::from_utf8(buf).unwrap(),
        "<h1>Hello World!</h1>\n",
    );
}
