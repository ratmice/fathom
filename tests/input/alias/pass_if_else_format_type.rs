#![cfg(test)]

use ddl_rt::{F64Be, FormatWriter, ReadError, ReadScope, U8};
use ddl_test_util::ddl::binary;

#[path = "../../snapshots/alias/pass_if_else_format_type.rs"]
mod fixture;

ddl_test_util::core_module!(
    FIXTURE,
    "../../snapshots/alias/pass_if_else_format_type.core.ddl"
);

#[test]
fn eof_inner() {
    let writer = FormatWriter::new(vec![]);

    let read_scope = ReadScope::new(writer.buffer());
    let singleton = read_scope.read::<fixture::Test>();

    match singleton {
        Err(ReadError::Eof(_)) => {}
        Err(err) => panic!("eof error expected, found: {:?}", err),
        Ok(_) => panic!("error expected, found: Ok(_)"),
    }

    // TODO: Check remaining
}

#[test]
fn valid_test() {
    let mut writer = FormatWriter::new(vec![]);
    writer.write::<F64Be>(23.64e10); // Test::inner

    let read_scope = ReadScope::new(writer.buffer());
    let singleton = read_scope.read::<fixture::Test>().unwrap();
    let mut read_context = binary::read::Context::new(read_scope.reader());

    let test = binary::read::read_module_item(&mut read_context, &FIXTURE, &"Test").unwrap();
    match singleton.inner() {
        fixture::Enum0::True(inner) => {
            assert_eq!(inner, 23.64e10);
            assert_eq!(test, binary::Term::F64(inner.into()));
        }
        _ => panic!("expected `Enum0::True(_)`"),
    }

    // TODO: Check remaining
}

#[test]
fn valid_test_trailing() {
    let mut writer = FormatWriter::new(vec![]);
    writer.write::<F64Be>(781.453298); // Test::inner
    writer.write::<U8>(42);

    let read_scope = ReadScope::new(writer.buffer());
    let singleton = read_scope.read::<fixture::Test>().unwrap();
    let mut read_context = binary::read::Context::new(read_scope.reader());

    let test = binary::read::read_module_item(&mut read_context, &FIXTURE, &"Test").unwrap();
    match singleton.inner() {
        fixture::Enum0::True(inner) => {
            assert_eq!(inner, 781.453298);
            assert_eq!(test, binary::Term::F64(inner.into()));
        }
        _ => panic!("expected `Enum0::True(_)`"),
    }

    // TODO: Check remaining
}