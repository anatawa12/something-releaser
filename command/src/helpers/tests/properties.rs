fn run_parse_and_print(str: &str, value: PropertiesFile) {
    let file = PropertiesFile::parse(str).unwrap();
    assert_eq!(file, value);
    let written = {
        let mut str = Vec::<u8>::with_capacity(str.len());
        file.write(&mut str).unwrap();
        String::from_utf8(str).unwrap()
    };
    assert_eq!(str, written);
}

#[test]
fn empty() {
    run_parse_and_print("", PropertiesFile { body: vec![] });
}

#[test]
fn empty_with_new_line() {
    run_parse_and_print(
        "\n",
        PropertiesFile {
            body: vec![SkipLine("")],
        },
    );
}

#[test]
fn blank() {
    run_parse_and_print(
        "    \n",
        PropertiesFile {
            body: vec![SkipLine("    ")],
        },
    );
}

#[test]
fn normal_kvp() {
    run_parse_and_print(
        "hello=world\n",
        PropertiesFile {
            body: vec![ElemKeyValuePair(KeyValuePair {
                key_parsed: "hello".to_owned(),
                blank: "",
                key: Some("hello"),
                separator: "=",
                value: Parsed(vec![BlankValuePair {
                    blank: "",
                    value: "world",
                }]),
            })],
        },
    );
}

#[test]
fn no_sep_kvp() {
    run_parse_and_print(
        "hello\\\nworld\n",
        PropertiesFile {
            body: vec![ElemKeyValuePair(KeyValuePair {
                key_parsed: "hello".to_owned(),
                blank: "",
                key: Some("hello"),
                separator: "",
                value: Parsed(vec![
                    BlankValuePair {
                        blank: "",
                        value: "",
                    },
                    BlankValuePair {
                        blank: "",
                        value: "world",
                    },
                ]),
            })],
        },
    );
}

#[test]
fn sep_with_space_kvp() {
    run_parse_and_print(
        "hello  =  world\n",
        PropertiesFile {
            body: vec![ElemKeyValuePair(KeyValuePair {
                key_parsed: "hello".to_owned(),
                blank: "",
                key: Some("hello"),
                separator: "  =  ",
                value: Parsed(vec![BlankValuePair {
                    blank: "",
                    value: "world",
                }]),
            })],
        },
    );
}

#[test]
fn some_seps_space_kvp() {
    run_parse_and_print(
        "hello:world\n",
        PropertiesFile {
            body: vec![ElemKeyValuePair(KeyValuePair {
                key_parsed: "hello".to_owned(),
                blank: "",
                key: Some("hello"),
                separator: ":",
                value: Parsed(vec![BlankValuePair {
                    blank: "",
                    value: "world",
                }]),
            })],
        },
    );
}

#[test]
fn comment_line() {
    run_parse_and_print(
        "!comment\n",
        PropertiesFile {
            body: vec![SkipLine("!comment")],
        },
    );
    run_parse_and_print(
        "#comment\n",
        PropertiesFile {
            body: vec![SkipLine("#comment")],
        },
    );
}

#[test]
fn complicated() {
    run_parse_and_print(
        "org.gradle.jvmargs=-Xmx1024m
# channel
#
# debug, snapshot, public
CHANNEL=snapshot
#
# versions
#
VERSIONS=1.5.7\\
     ,1.5.6\\
     ,1.5.5
#
# other flags
#
# enable optifine debugging
ENABLE_\\:OPTIFINE=false
",
        PropertiesFile {
            body: vec![
                ElemKeyValuePair(KeyValuePair {
                    key_parsed: "org.gradle.jvmargs".to_owned(),
                    blank: "",
                    key: Some("org.gradle.jvmargs"),
                    separator: "=",
                    value: Parsed(vec![BlankValuePair {
                        blank: "",
                        value: "-Xmx1024m",
                    }]),
                }),
                SkipLine("# channel"),
                SkipLine("#"),
                SkipLine("# debug, snapshot, public"),
                ElemKeyValuePair(KeyValuePair {
                    key_parsed: "CHANNEL".to_owned(),
                    blank: "",
                    key: Some("CHANNEL"),
                    separator: "=",
                    value: Parsed(vec![BlankValuePair {
                        blank: "",
                        value: "snapshot",
                    }]),
                }),
                SkipLine("#"),
                SkipLine("# versions"),
                SkipLine("#"),
                ElemKeyValuePair(KeyValuePair {
                    key_parsed: "VERSIONS".to_owned(),
                    blank: "",
                    key: Some("VERSIONS"),
                    separator: "=",
                    value: Parsed(vec![
                        BlankValuePair {
                            blank: "",
                            value: "1.5.7",
                        },
                        BlankValuePair {
                            blank: "     ",
                            value: ",1.5.6",
                        },
                        BlankValuePair {
                            blank: "     ",
                            value: ",1.5.5",
                        },
                    ]),
                }),
                SkipLine("#"),
                SkipLine("# other flags"),
                SkipLine("#"),
                SkipLine("# enable optifine debugging"),
                ElemKeyValuePair(KeyValuePair {
                    key_parsed: "ENABLE_:OPTIFINE".to_owned(),
                    blank: "",
                    key: Some("ENABLE_\\:OPTIFINE"),
                    separator: "=",
                    value: Parsed(vec![BlankValuePair {
                        blank: "",
                        value: "false",
                    }]),
                }),
            ],
        },
    );
}
