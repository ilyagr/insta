use insta::assert_snapshot;

use crate::TestFiles;

// Fun fact: cargo insta test --workspace --accept -- allow_duplicates
// gives a different result than
// cargo insta test --workspace --accept -- test_allow_duplicates_loop
// for the same test.
// TODO: Deleteme, this exists mainly for `cargo fmt`
#[test]
fn test_allow_duplicates_deleteme() {
    let message = r##"
    Message "1" @
    "##;
    insta::allow_duplicates! {
        insta::assert_snapshot!(message, @r##"
        Message "1" @
    "##);    
    }
    let message = r##"
    Message "2" @
    "##;
    insta::allow_duplicates! {
    insta::assert_snapshot!(message, @r##"
    Message "2" @
    "##);
    }

    // Two assertions in one allow_duplicates!
    insta::allow_duplicates! {
        let message = r##"
        Message "3"
        Another line
    "##;
        insta::assert_snapshot!(message, @r##"
        Message "3"
        Another line
    "##);
    let message = r##"
       Message "4" @
    "##;
    insta::assert_snapshot!(message, @r##"
       Message "4" @
    "##);
    }
}

#[test]
fn test_allow_duplicates_onepass() {
    let test_project = TestFiles::new()
        .add_file(
            "Cargo.toml",
            r#"
[package]
name = "test_json_inline"
version = "0.1.0"
edition = "2021"

[dependencies]
insta = { path = '$PROJECT_PATH' }
"#
            .to_string(),
        )
        .add_file(
            "src/lib.rs",
            r####"
#[test]
fn test_allow_duplicates_1run() {
    let message = r##"
    Message "1" @
    "##;
    insta::allow_duplicates! {
        insta::assert_snapshot!(message, @r"");    
    }
    let message = r##"
    Message "2" @
    "##;
    insta::allow_duplicates! {
    insta::assert_snapshot!(message, @r"");
    }

    // Two assertions in one allow_duplicates!
    insta::allow_duplicates! {
        let message = r##"
        Message "3"
        Another line
    "##;
        insta::assert_snapshot!(message, @r"");
    let message = r##"
       Message "4" @
    "##;
    insta::assert_snapshot!(message, @r"");
    }
}
"####
                .to_string(),
        )
        .create_project();

    let output = test_project
        .insta_cmd()
        .args(["test", "--accept", "--", "--nocapture"])
        .output()
        .unwrap();

    assert!(&output.status.success());

    assert_snapshot!(test_project.diff("src/lib.rs"), @"");
}

#[test]
fn test_allow_duplicates_loop() {
    let test_project = TestFiles::new()
        .add_file(
            "Cargo.toml",
            r#"
[package]
name = "test_json_inline"
version = "0.1.0"
edition = "2021"

[dependencies]
insta = { path = '$PROJECT_PATH' }
"#
            .to_string(),
        )
        .add_file(
            "src/lib.rs",
            r####"
#[test]
fn test_allow_duplicates_10run() {
    for x in (0..10) {
        let message = r##"
        Message "1" @
        "##;
        insta::allow_duplicates! {
            insta::assert_snapshot!(message, @r"");    
        }
        let message = r##"
        Message "2" @
        "##;
        insta::allow_duplicates! {
        insta::assert_snapshot!(message, @r"");
        }

        // Two assertions in one allow_duplicates!
        insta::allow_duplicates! {
            let message = r##"
            Message "3"
            Another line
        "##;
            insta::assert_snapshot!(message, @r"");
        let message = r##"
           Message "4" @
        "##;
        insta::assert_snapshot!(message, @r"");
        }
    }
}
"####
                .to_string(),
        )
        .create_project();

    let output = test_project
        .insta_cmd()
        .args(["test", "--accept", "--", "--nocapture"])
        .output()
        .unwrap();

    assert!(&output.status.success());

    assert_snapshot!(test_project.diff("src/lib.rs"), @"");
}

// Sometimes, only one of two tests gets updated
#[test]
fn test_allow_duplicates_twotests() {
    let test_project = TestFiles::new()
        .add_file(
            "Cargo.toml",
            r#"
[package]
name = "test_json_inline"
version = "0.1.0"
edition = "2021"

[dependencies]
insta = { path = '$PROJECT_PATH' }
"#
            .to_string(),
        )
        .add_file(
            "src/lib.rs",
            r####"
#[test]
fn test_allow_duplicates_1run() {
    let message = r##"
    Message "1" @
    "##;
    insta::allow_duplicates! {
        insta::assert_snapshot!(message, @r"");    
    }
    let message = r##"
    Message "2" @
    "##;
    insta::allow_duplicates! {
    insta::assert_snapshot!(message, @r"");
    }

    // Two assertions in one allow_duplicates!
    insta::allow_duplicates! {
        let message = r##"
        Message "3"
        Another line
    "##;
        insta::assert_snapshot!(message, @r"");
    let message = r##"
       Message "4" @
    "##;
    insta::assert_snapshot!(message, @r"");
    }
}



#[test]
fn test_allow_duplicates_10run() {
    for x in (0..10) {
        let message = r##"
        Message "1" @
        "##;
        insta::allow_duplicates! {
            insta::assert_snapshot!(message, @r"");    
        }
        let message = r##"
        Message "2" @
        "##;
        insta::allow_duplicates! {
        insta::assert_snapshot!(message, @r"");
        }

        // Two assertions in one allow_duplicates!
        insta::allow_duplicates! {
            let message = r##"
            Message "3"
            Another line
        "##;
            insta::assert_snapshot!(message, @r"");
        let message = r##"
           Message "4" @
        "##;
        insta::assert_snapshot!(message, @r"");
        }
    }
}
"####
                .to_string(),
        )
        .create_project();

    let output = test_project
        .insta_cmd()
        .args(["test", "--accept", "--", "--nocapture"])
        .output()
        .unwrap();

    assert!(&output.status.success());

    assert_snapshot!(test_project.diff("src/lib.rs"), @r###"
    --- Original: src/lib.rs
    +++ Updated: src/lib.rs
    @@ -5,13 +5,13 @@
         Message "1" @
         "##;
         insta::allow_duplicates! {
    -        insta::assert_snapshot!(message, @r"");    
    +        insta::assert_snapshot!(message, @r#"Message "1" @"#);    
         }
         let message = r##"
         Message "2" @
         "##;
         insta::allow_duplicates! {
    -    insta::assert_snapshot!(message, @r"");
    +    insta::assert_snapshot!(message, @r#"Message "2" @"#);
         }
     
         // Two assertions in one allow_duplicates!
    @@ -24,7 +24,10 @@
         let message = r##"
            Message "4" @
         "##;
    -    insta::assert_snapshot!(message, @r"");
    +    insta::assert_snapshot!(message, @r#"
    +    Message "3"
    +    Another line
    +    "#);r#"Message "4" @"#
         }
     }
     
    @@ -37,13 +40,13 @@
             Message "1" @
             "##;
             insta::allow_duplicates! {
    -            insta::assert_snapshot!(message, @r"");    
    +            insta::assert_snapshot!(message, @r#"Message "1" @"#Message "1" @"#Message "1" @"#Message "1" @"#Message "1" @"#Message "1" @"#Message "1" @"#Message "1" @"#Message "1" @"#Message "1" @"#);    
             }
             let message = r##"
             Message "2" @
             "##;
             insta::allow_duplicates! {
    -        insta::assert_snapshot!(message, @r"");
    +        insta::assert_snapshot!(message, @r#"Message "2" @"#Message "2" @"#Message "2" @"#Message "2" @"#Message "2" @"#Message "2" @"#Message "2" @"#Message "2" @"#Message "2" @"#Message "2" @"#);
             }
     
             // Two assertions in one allow_duplicates!
    @@ -56,7 +59,10 @@
             let message = r##"
                Message "4" @
             "##;
    -        insta::assert_snapshot!(message, @r"");
    +        insta::assert_snapshot!(message, @r#"
    +        Message "3"
    +        Another line
    +        "#);r#"Message "4" @"#r#"Message "r#"Message "4" @"#Message "4" @"#Message "4" @"#Message "4" @"#Message "4" @"#Message "4" @"#Message "4" @"#Message "4" @"#@"#
             }
         }
     }
    "###);
}

#[test]
fn test_allow_duplicates_test_case() {
    let test_project = TestFiles::new()
        .add_file(
            "Cargo.toml",
            r#"
[package]
name = "test_json_inline"
version = "0.1.0"
edition = "2021"

[dependencies]
insta = { path = '$PROJECT_PATH' }
test-case = "3.3.1"
"#
            .to_string(),
        )
        .add_file(
            "src/lib.rs",
            r####"
use test_case::test_case;

#[test_case(1; "run 1")]
#[test_case(2; "run 2")]
#[test_case(3; "run 3")]
fn test_allow_duplicates_3run(_index: usize) {
    let message = r##"
    Message "1" @
    "##;
    insta::allow_duplicates! {
        insta::assert_snapshot!(message, @r"");    
    }
    let message = r##"
    Message "2" @
    "##;
    insta::allow_duplicates! {
    insta::assert_snapshot!(message, @r"");
    }

    // Two assertions in one allow_duplicates!
    insta::allow_duplicates! {
        let message = r##"
        Message "3"
        Another line
    "##;
        insta::assert_snapshot!(message, @r"");
    let message = r##"
       Message "4" @
    "##;
    insta::assert_snapshot!(message, @r"");
    }
}
"####
                .to_string(),
        )
        .create_project();

    let output = test_project
        .insta_cmd()
        .args(["test", "--accept", "--", "--nocapture"])
        .output()
        .unwrap();

    assert!(&output.status.success());

    assert_snapshot!(test_project.diff("src/lib.rs"), @r###"
    --- Original: src/lib.rs
    +++ Updated: src/lib.rs
    @@ -9,13 +9,13 @@
         Message "1" @
         "##;
         insta::allow_duplicates! {
    -        insta::assert_snapshot!(message, @r"");    
    +        insta::assert_snapshot!(message, @r#"Message "1" @"#Message "1" @"#Message "1" @"#);    
         }
         let message = r##"
         Message "2" @
         "##;
         insta::allow_duplicates! {
    -    insta::assert_snapshot!(message, @r"");
    +    insta::assert_snapshot!(message, @r#"Message "2" @"#Message "2" @"#Message "2" @"#);
         }
     
         // Two assertions in one allow_duplicates!
    @@ -28,6 +28,9 @@
         let message = r##"
            Message "4" @
         "##;
    -    insta::assert_snapshot!(message, @r"");
    +    insta::assert_snapshot!(message, @r#"
    +    Message "3"
    +    Another line
    +    "#);r#"Message "4" @"#r#"Message "r#"Message "4" @"#@"#
         }
     }
    "###);
}
