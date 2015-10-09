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
}
