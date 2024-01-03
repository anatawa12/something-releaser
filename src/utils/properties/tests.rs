use super::*;

mod parse_and_write {
    use super::*;

    fn parse_and_write(input: &str, expected: Vec<Element>) {
        let parsed = PropertiesFile::from_str(input).unwrap();
        assert_eq!(parsed.elements, expected);
        let written = parsed.to_string();
        assert_eq!(input, written);
    }

    #[test]
    fn empty() {
        parse_and_write("", vec![]);
    }

    #[test]
    fn new_line_only() {
        parse_and_write("\n", vec![Element::BlankOrComment("".into())]);
    }

    #[test]
    fn blank() {
        parse_and_write("      \n", vec![Element::BlankOrComment("      ".into())]);
    }

    #[test]
    fn simple_key_value() {
        parse_and_write(
            "hello=world\n",
            vec![Element::KeyValuePair(KeyValuePair {
                blank_before_key: "".into(),
                key: Some("hello".into()),
                key_parsed: "hello".into(),
                separator: "=".into(),
                value: Value::Parsed {
                    lines: vec![
                        ParsedValueLine {
                            blank: "".into(),
                            line: "world".into(),
                            parsed: "world".into(),
                        },
                    ],
                },
            })],
        );
    }

    #[test]
    fn newline_separator() {
        parse_and_write(
            "hello\\\nworld\n",
            vec![Element::KeyValuePair(KeyValuePair {
                blank_before_key: "".into(),
                key: Some("hello".into()),
                key_parsed: "hello".into(),
                separator: "".into(),
                value: Value::Parsed {
                    lines: vec![
                        ParsedValueLine {
                            blank: "".into(),
                            line: "".into(),
                            parsed: "".into(),
                        },
                        ParsedValueLine {
                            blank: "".into(),
                            line: "world".into(),
                            parsed: "world".into(),
                        },
                    ],
                },
            })],
        );
    }

    #[test]
    fn separator_with_space() {
        parse_and_write(
            "hello = world\n",
            vec![Element::KeyValuePair(KeyValuePair {
                blank_before_key: "".into(),
                key: Some("hello".into()),
                key_parsed: "hello".into(),
                separator: " = ".into(),
                value: Value::Parsed {
                    lines: vec![
                        ParsedValueLine {
                            blank: "".into(),
                            line: "world".into(),
                            parsed: "world".into(),
                        },
                    ],
                },
            })],
        );
    }

    #[test]
    fn colon_separated() {
        parse_and_write(
            "hello:world\n",
            vec![Element::KeyValuePair(KeyValuePair {
                blank_before_key: "".into(),
                key: Some("hello".into()),
                key_parsed: "hello".into(),
                separator: ":".into(),
                value: Value::Parsed {
                    lines: vec![
                        ParsedValueLine {
                            blank: "".into(),
                            line: "world".into(),
                            parsed: "world".into(),
                        },
                    ],
                },
            })],
        );
    }

    #[test]
    fn comment_line() {
        parse_and_write(
            "! comment\n",
            vec![Element::BlankOrComment("! comment".into())],
        );

        parse_and_write(
            "# comment\n",
            vec![Element::BlankOrComment("# comment".into())],
        );

        parse_and_write(
            "   #comment\n",
            vec![Element::BlankOrComment("   #comment".into())],
        );
    }

    #[test]
    fn complicated() {
        parse_and_write(
            r##"org.gradle.jvmargs=-Xmx1024m
# channel
#
# debug, snapshot, public
CHANNEL=snapshot
#
# versions
#
VERSIONS=1.5.7\
     ,1.5.6\
     ,1.5.5
#
# other flags
#
# enable optifine debugging
ENABLE_\:OPTIFINE=false
"##,
            vec![
                Element::KeyValuePair(KeyValuePair {
                    blank_before_key: "".into(),
                    key: Some("org.gradle.jvmargs".into()),
                    key_parsed: "org.gradle.jvmargs".into(),
                    separator: "=".into(),
                    value: Value::Parsed {
                        lines: vec![
                            ParsedValueLine {
                                blank: "".into(),
                                line: "-Xmx1024m".into(),
                                parsed: "-Xmx1024m".into(),
                            },
                        ],
                    },
                }),
                Element::BlankOrComment("# channel".into()),
                Element::BlankOrComment("#".into()),
                Element::BlankOrComment("# debug, snapshot, public".into()),
                Element::KeyValuePair(KeyValuePair {
                    blank_before_key: "".into(),
                    key: Some("CHANNEL".into()),
                    key_parsed: "CHANNEL".into(),
                    separator: "=".into(),
                    value: Value::Parsed {
                        lines: vec![
                            ParsedValueLine {
                                blank: "".into(),
                                line: "snapshot".into(),
                                parsed: "snapshot".into(),
                            },
                        ],
                    },
                }),
                Element::BlankOrComment("#".into()),
                Element::BlankOrComment("# versions".into()),
                Element::BlankOrComment("#".into()),
                Element::KeyValuePair(KeyValuePair {
                    blank_before_key: "".into(),
                    key: Some("VERSIONS".into()),
                    key_parsed: "VERSIONS".into(),
                    separator: "=".into(),
                    value: Value::Parsed {
                        lines: vec![
                            ParsedValueLine {
                                blank: "".into(),
                                line: "1.5.7".into(),
                                parsed: "1.5.7".into(),
                            },
                            ParsedValueLine {
                                blank: "     ".into(),
                                line: ",1.5.6".into(),
                                parsed: ",1.5.6".into(),
                            },
                            ParsedValueLine {
                                blank: "     ".into(),
                                line: ",1.5.5".into(),
                                parsed: ",1.5.5".into(),
                            },
                        ],
                    },
                }),
                Element::BlankOrComment("#".into()),
                Element::BlankOrComment("# other flags".into()),
                Element::BlankOrComment("#".into()),
                Element::BlankOrComment("# enable optifine debugging".into()),
                Element::KeyValuePair(KeyValuePair {
                    blank_before_key: "".into(),
                    key: Some("ENABLE_\\:OPTIFINE".into()),
                    key_parsed: "ENABLE_:OPTIFINE".into(),
                    separator: "=".into(),
                    value: Value::Parsed {
                        lines: vec![
                            ParsedValueLine {
                                blank: "".into(),
                                line: "false".into(),
                                parsed: "false".into(),
                            },
                        ],
                    },
                }),
            ],
        )
    }
}

mod parse_escape_sequence {
    use super::*;
    use parsing::EscapeParser;

    fn tester(line: &str, end_with_backslash: bool, expected: &[(bool, char, &str)]) {
        let mut parser = EscapeParser::new(line.chars(), 0);
        for (index, tuple) in expected.iter().enumerate() {
            let (escape, parsed, raw) = parser.next_char().unwrap().unwrap();
            assert_eq!(escape, tuple.0, "at {index}");
            assert_eq!(parsed, tuple.1, "at {index}");
            assert_eq!(raw, tuple.2, "at {index}");
        }
        assert!(parser.next_char().unwrap().is_none());
        assert_eq!(parser.ends_with_backslash, end_with_backslash);
    }

    #[test]
    fn simple_line() {
        tester("", false, &[]);
        tester("\\", true, &[]);

        tester("hello", false, &[
            (false, 'h', "h"),
            (false, 'e', "e"),
            (false, 'l', "l"),
            (false, 'l', "l"),
            (false, 'o', "o"),
        ]);

        tester("hello\\", true, &[
            (false, 'h', "h"),
            (false, 'e', "e"),
            (false, 'l', "l"),
            (false, 'l', "l"),
            (false, 'o', "o"),
        ]);
    }

    #[test]
    fn c_style_escape() {
        tester("\\t\\r\\n\\f", false, &[
            (true, '\t', "\\t"),
            (true, '\r', "\\r"),
            (true, '\n', "\\n"),
            (true, '\x0C', "\\f"),
        ]);
        tester("\\t\\r\\n\\f\\", true, &[
            (true, '\t', "\\t"),
            (true, '\r', "\\r"),
            (true, '\n', "\\n"),
            (true, '\x0C', "\\f"),
        ]);
    }

    #[test]
    fn same_char_escape() {
        tester("\\a\\b\\:\\!\\ ", false, &[
            (true, 'a', "\\a"),
            (true, 'b', "\\b"),
            (true, ':', "\\:"),
            (true, '!', "\\!"),
            (true, ' ', "\\ "),
        ]);
        tester("\\a\\b\\:\\!\\ \\", true, &[
            (true, 'a', "\\a"),
            (true, 'b', "\\b"),
            (true, ':', "\\:"),
            (true, '!', "\\!"),
            (true, ' ', "\\ "),
        ]);
    }
}

mod get_value {
    use super::*;

    #[test]
    fn simple() {
        let properties = PropertiesFile::from_str("hello=world").unwrap();
        assert_eq!(properties.get("hello"), Some("world".into()));
        assert_eq!(properties.get("world"), None);
    }

    #[test]
    fn multi_line() {
        let properties = PropertiesFile::from_str("hello=world\\\n world").unwrap();
        assert_eq!(properties.get("hello"), Some("worldworld".into()));
        assert_eq!(properties.get("world"), None);
    }
}

mod set_value {
    use super::*;

    #[test]
    fn write_new_value() {
        let mut properties = PropertiesFile::from_str("hello=world").unwrap();
        properties.set("world", "hello".to_owned());
        assert_eq!(properties.get("world"), Some("hello".into()));
        assert_eq!(properties.to_string(), "hello=world\nworld=hello\n");
    }

    #[test]
    fn write_existing_value() {
        let mut properties = PropertiesFile::from_str("hello=world").unwrap();
        properties.set("hello", "hello".to_owned());
        assert_eq!(properties.get("hello"), Some("hello".into()));
        assert_eq!(properties.to_string(), "hello=hello\n");
    }

    #[test]
    fn key_with_space_at_first() {
        let mut properties = PropertiesFile::from_str("").unwrap();
        properties.set(" hello", "hello".to_owned());
        assert_eq!(properties.get(" hello"), Some("hello".into()));
        assert_eq!(properties.to_string(), "\\ hello=hello\n");
    }
}
