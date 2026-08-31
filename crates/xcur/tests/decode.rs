use xcur::Xcursor;

#[test]
fn default() {
    let bytes = include_bytes!("./data/default");
    let xcursor = Xcursor::try_from(bytes.as_slice()).expect("failed to parse default cursor");

    assert_eq!(xcursor.images().len(), 66);
    assert_eq!(xcursor.comments().len(), 0);
}

#[test]
fn text() {
    let bytes = include_bytes!("./data/text");
    let xcursor = Xcursor::try_from(bytes.as_slice()).expect("failed to parse default cursor");
    println!("Images: {:?}", xcursor.images());
}
