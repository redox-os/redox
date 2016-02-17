pub fn meta_test_woah_fail() -> bool {
    test!(true == false);
    test!(false);
    fail!();
}

pub fn meta_test_woah() -> bool {
    test!(true == true);
    test!(true);
    succ!();
}
