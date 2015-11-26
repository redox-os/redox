/// A module for permission units
pub mod permission_unit;
/// A module for checking string (wildcard) matches
pub mod str_match;
/// A module for permissions
pub mod permission;

#[test]
fn test() {
    use str_match::*;
    use permission_unit::*;
    use permission::*;
    // Test string matches (wildcard chars)
    assert!(str_match("hey*hey", "heyabchey"));
    assert!(str_match("hey\\*hey*", "hey*heycatsarefunny"));
    // Test permission units
    assert!(PermissionUnit::from_str("rw=hey").read());
    assert!(!PermissionUnit::from_str("r=hey").write());
    assert!(PermissionUnit::from_str("r=hey").read_foc());
    assert!(PermissionUnit::from_str("r=file:home/*")
                .applies(&PermissionUnit::from_str("R=file:home/lal")));
    assert!(PermissionUnit::from_str("R=file:home/lal").read_foc);
    assert!(PermissionUnit::from_str("r=file:home/lal").read_foc());
    assert!(!PermissionUnit::from_str("RW=http:*").read());
    // Test permissions
    assert!(Permission::from_str("rw=file:home/*-rw=file:veryimportant")
                .test(PermissionUnit::from_str("rw=file:home/lal")));
    assert!(Permission::from_str("rw=file:home/*-rw=file:veryimportant")
                .test(PermissionUnit::from_str("rw=file:home/veryimportant")));
    assert!(Permission::from_str("rw=i\\+can\\+do\\+like\\+this")
                .test(PermissionUnit::from_str("rw=file:i+can+do+like+this")));

    // assert!(!Permission::from_str("RW=http:*").test(PermissionUnit::from_str("rw=http://google.com")));
    // TODO: Failes when using uppercase RW
}
