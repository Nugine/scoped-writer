use scoped_writer::g;
use scoped_writer::scoped;
use scoped_writer::with;

use std::fs;
use std::io::Read as _;
use std::io::Seek;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;
use std::path::Path;

#[test]
fn test_panic() {
    let res = with(|w| writeln!(w));
    assert!(res.is_none());

    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();

    let res = catch_unwind(AssertUnwindSafe(|| {
        scoped(&mut buf1, || {
            scoped(&mut buf2, || {
                panic!();
            });
        });
    }));

    assert!(res.is_err());
    assert!(buf1.is_empty());
    assert!(buf2.is_empty());

    let res = with(|w| writeln!(w));
    assert!(res.is_none());
}

#[should_panic(expected = "reentrancy detected")]
#[test]
fn test_reentrancy_1() {
    let mut buf = Vec::new();

    scoped(&mut buf, || {
        with(|w1| {
            with(|w2| {
                writeln!(w1, "Hello").unwrap();
                writeln!(w2, "World").unwrap();
            })
        })
    });
}

#[should_panic(expected = "reentrancy detected")]
#[test]
fn test_reentrancy_2() {
    let mut buf = Vec::new();

    scoped(&mut buf, || {
        with(|w| {
            scoped(w, || {
                //
            })
        })
    });
}

#[test]
fn test_empty() {
    let res = with(|w| writeln!(w, "hello"));
    assert!(res.is_none());
}

#[test]
fn test_vec() {
    let mut buf = Vec::new();

    scoped(&mut buf, || {
        let x = 42;

        g!("Hello");
        g!();
        g!("The answer is {}", x);
        g!("The answer is {x}");
        g!("The answer is {num}", num = x);
    });

    assert_eq!(
        String::from_utf8(buf).unwrap(),
        concat!(
            "Hello\n",
            "\n",
            "The answer is 42\n",
            "The answer is 42\n",
            "The answer is 42\n"
        )
    );
}

#[test]
fn test_file() {
    let root = Path::new(concat!(env!("CARGO_TARGET_TMPDIR"), "/tests_basic"));
    fs::create_dir_all(root).unwrap();

    let file_path = root.join("file.txt");

    fs::remove_file(&file_path).ok();

    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file_path)
        .unwrap();

    scoped(&mut file, || g(["Hello", "", "World"]));

    {
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello\n\nWorld\n");
    }

    {
        let mut content = String::new();
        file.rewind().unwrap();
        file.read_to_string(&mut content).unwrap();
        assert_eq!(content, "Hello\n\nWorld\n");
    }
}

#[test]
fn test_match_arm() {
    match None::<u32> {
        Some(x) => {
            println!("{x}");
        }
        None => g!(),
    }
}
