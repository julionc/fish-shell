use crate::builtins::prelude::*;
use crate::builtins::test::test as builtin_test;
use crate::io::{IoChain, OutputStream};
use crate::tests::prelude::*;

fn run_one_test_test_mbracket(expected: i32, lst: &[&str], bracket: bool) -> bool {
    let parser = TestParser::new();
    let mut argv = Vec::new();
    if bracket {
        argv.push(L!("[").to_owned());
    } else {
        argv.push(L!("test").to_owned());
    }
    for s in lst {
        argv.push(WString::from_str(s));
    }
    if bracket {
        argv.push(L!("]").to_owned())
    };

    // Convert to &[&wstr].
    let mut argv = argv.iter().map(|s| s.as_ref()).collect::<Vec<_>>();
    let mut out = OutputStream::Null;
    let mut err = OutputStream::Null;
    let io_chain = IoChain::new();
    let mut streams = IoStreams::new(&mut out, &mut err, &io_chain);

    let result = builtin_test(&parser, &mut streams, &mut argv).builtin_status_code();

    if result != expected {
        eprintf!(
            "expected builtin_test() to return %s, got %s\n",
            expected.to_string(),
            result.to_string()
        );
    }
    result == expected
}

fn run_test_test(expected: i32, lst: &[&str]) -> bool {
    let nobracket = run_one_test_test_mbracket(expected, lst, false);
    let bracket = run_one_test_test_mbracket(expected, lst, true);
    assert_eq!(nobracket, bracket);
    nobracket
}

fn test_test_brackets() {
    // Ensure [ knows it needs a ].
    let parser = TestParser::new();

    let mut out = OutputStream::Null;
    let mut err = OutputStream::Null;
    let io_chain = IoChain::new();
    let mut streams = IoStreams::new(&mut out, &mut err, &io_chain);

    let args1 = &mut [L!("["), L!("foo")];
    assert_eq!(
        builtin_test(&parser, &mut streams, args1),
        Err(STATUS_INVALID_ARGS)
    );

    let args2 = &mut [L!("["), L!("foo"), L!("]")];
    assert_eq!(builtin_test(&parser, &mut streams, args2), Ok(SUCCESS));

    let args3 = &mut [L!("["), L!("foo"), L!("]"), L!("bar")];
    assert_eq!(
        builtin_test(&parser, &mut streams, args3),
        Err(STATUS_INVALID_ARGS)
    );
}

#[rustfmt::skip]
fn test_test() {
    assert!(run_test_test(0, &["5", "-ne", "6"]));
    assert!(run_test_test(0, &["5", "-eq", "5"]));
    assert!(run_test_test(0, &["0", "-eq", "0"]));
    assert!(run_test_test(0, &["-1", "-eq", "-1"]));
    assert!(run_test_test(0, &["1", "-ne", "-1"]));
    assert!(run_test_test(1, &[" 2 ", "-ne", "2"]));
    assert!(run_test_test(0, &[" 2", "-eq", "2"]));
    assert!(run_test_test(0, &["2 ", "-eq", "2"]));
    assert!(run_test_test(0, &[" 2 ", "-eq", "2"]));
    assert!(run_test_test(2, &[" 2x", "-eq", "2"]));
    assert!(run_test_test(2, &["", "-eq", "0"]));
    assert!(run_test_test(2, &["", "-ne", "0"]));
    assert!(run_test_test(2, &["  ", "-eq", "0"]));
    assert!(run_test_test(2, &["  ", "-ne", "0"]));
    assert!(run_test_test(2, &["x", "-eq", "0"]));
    assert!(run_test_test(2, &["x", "-ne", "0"]));
    assert!(run_test_test(1, &["-1", "-ne", "-1"]));
    assert!(run_test_test(0, &["abc", "!=", "def"]));
    assert!(run_test_test(1, &["abc", "=", "def"]));
    assert!(run_test_test(0, &["5", "-le", "10"]));
    assert!(run_test_test(0, &["10", "-le", "10"]));
    assert!(run_test_test(1, &["20", "-le", "10"]));
    assert!(run_test_test(0, &["-1", "-le", "0"]));
    assert!(run_test_test(1, &["0", "-le", "-1"]));
    assert!(run_test_test(0, &["15", "-ge", "10"]));
    assert!(run_test_test(0, &["15", "-ge", "10"]));
    assert!(run_test_test(1, &["!", "15", "-ge", "10"]));
    assert!(run_test_test(0, &["!", "!", "15", "-ge", "10"]));

    assert!(run_test_test(0, &[
        "(", "-d", "/", ")",
        "-o",
        "(", "!", "-d", "/", ")",
    ]));

    assert!(run_test_test(0, &["0", "-ne", "1", "-a", "0", "-eq", "0"]));
    assert!(run_test_test(0, &["0", "-ne", "1", "-a", "-n", "5"]));
    assert!(run_test_test(0, &["-n", "5", "-a", "10", "-gt", "5"]));
    assert!(run_test_test(0, &["-n", "3", "-a", "-n", "5"]));

    // Test precedence:
    //      '0 == 0 || 0 == 1 && 0 == 2'
    //  should be evaluated as:
    //      '0 == 0 || (0 == 1 && 0 == 2)'
    //  and therefore true. If it were
    //      '(0 == 0 || 0 == 1) && 0 == 2'
    //  it would be false.
    assert!(run_test_test(0, &["0", "=", "0", "-o", "0", "=", "1", "-a", "0", "=", "2"]));
    assert!(run_test_test(0, &["-n", "5", "-o", "0", "=", "1", "-a", "0", "=", "2"]));
    assert!(run_test_test(1, &["(", "0", "=", "0", "-o", "0", "=", "1", ")", "-a", "0", "=", "2"]));
    assert!(run_test_test(0, &["0", "=", "0", "-o", "(", "0", "=", "1", "-a", "0", "=", "2", ")"]));

    // A few lame tests for permissions; these need to be a lot more complete.
    assert!(run_test_test(0, &["-e", "/bin/ls"]));
    assert!(run_test_test(1, &["-e", "/bin/ls_not_a_path"]));
    assert!(run_test_test(0, &["-x", "/bin/ls"]));
    assert!(run_test_test(1, &["-x", "/bin/ls_not_a_path"]));
    assert!(run_test_test(0, &["-d", "/bin/"]));
    assert!(run_test_test(1, &["-d", "/bin/ls"]));

    // This failed at one point.
    assert!(run_test_test(1, &["-d", "/bin", "-a", "5", "-eq", "3"]));
    assert!(run_test_test(0, &["-d", "/bin", "-o", "5", "-eq", "3"]));
    assert!(run_test_test(0,&["-d", "/bin", "-a", "!", "5", "-eq", "3"]));

    // We didn't properly handle multiple "just strings" either.
    assert!(run_test_test(0, &["foo"]));
    assert!(run_test_test(0, &["foo", "-a", "bar"]));

    // These should be errors.
    assert!(run_test_test(1, &["foo", "bar"]));
    assert!(run_test_test(1, &["foo", "bar", "baz"]));

    // This crashed.
    assert!(run_test_test(1, &["1", "=", "1", "-a", "=", "1"]));

    // Make sure we can treat -S as a parameter instead of an operator.
    // https://github.com/fish-shell/fish-shell/issues/601
    assert!(run_test_test(0, &["-S", "=", "-S"]));
    assert!(run_test_test(1, &["!", "!", "!", "A"]));

    // Verify that 1. doubles are treated as doubles, and 2. integers that cannot be represented as
    // doubles are still treated as integers.
    assert!(run_test_test(0, &["4611686018427387904", "-eq", "4611686018427387904"]));
    assert!(run_test_test(0, &["4611686018427387904.0", "-eq", "4611686018427387904.0"]));
    assert!(run_test_test(0, &["4611686018427387904.00000000000000001", "-eq", "4611686018427387904.0"]));
    assert!(run_test_test(1, &["4611686018427387904", "-eq", "4611686018427387905"]));
    assert!(run_test_test(0, &["-4611686018427387904", "-ne", "4611686018427387904"]));
    assert!(run_test_test(0, &["-4611686018427387904", "-le", "4611686018427387904"]));
    assert!(run_test_test(1, &["-4611686018427387904", "-ge", "4611686018427387904"]));
    assert!(run_test_test(1, &["4611686018427387904", "-gt", "4611686018427387904"]));
    assert!(run_test_test(0, &["4611686018427387904", "-ge", "4611686018427387904"]));

    // test out-of-range numbers
    assert!(run_test_test(2, &["99999999999999999999999999", "-ge", "1"]));
    assert!(run_test_test(2, &["1", "-eq", "-99999999999999999999999999.9"]));
}

#[test]
#[serial]
fn test_test_builtin() {
    let _cleanup = test_init();
    test_test_brackets();
    test_test();
}
