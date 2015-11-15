#[test]
fn get_slice_array_test() {
    let array = [1, 2, 3, 4, 5];
    assert_eq!(array.get_slice(2, None), array[2..]);
    assert_eq!(array.get_slice(2, array.len()), array[2..array.len()]);
    assert_eq!(array.get_slice(None, 2), array[..2]);
    assert_eq!(array.get_slice(0, 2), array[0..2]);
    assert_eq!(array.get_slice(1, array.len()), array[1..array.len()]);
    assert_eq!(array.get_slice(1, array.len() + 1), array[1..array.len()]);
    assert_eq!(array.get_slice(array.len(), array.len()),
               array[array.len()..array.len()]);
    assert_eq!(array.get_slice(array.len() + 2, array.len() + 2),
               array[array.len()..array.len()]);
}
