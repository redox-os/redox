pub fn test() -> bool {
    use common::slice::GetSlice;
    let array = [1, 2, 3, 4, 5];

    test!(array.get_slice(100..100) == &[]);
    test!(array.get_slice(..100) == &array);
    test!(array.get_slice(1..) == &array[1..]);
    test!(array.get_slice(1..2) == &[2]);
    test!(array.get_slice(3..5) == &[4, 5]);
    test!(array.get_slice(3..7) == &[4, 5]);
    test!(array.get_slice(3..4) == &[4]);
    test!(array.get_slice(4..2) == &[]);
    test!(array.get_slice(4..1) == &[]);
    test!(array.get_slice(20..) == &[]);
    //test!(array.get_slice(..) == &array);
    succ!();
}
