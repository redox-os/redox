use core::*;
use common::*;

type Map = HashMap<String, Value>;

pub enum TomlType {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Datetime(String),
    Array(Array),
    Map(Map),
}

