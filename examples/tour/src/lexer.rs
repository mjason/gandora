
#[cfg(test)]
mod heredoc_tests {
    use super::*;

    #[test]
    fn lexes_heredoc_strings() {
        let toks = Lexer::new("<test>", "\"\"\"\nline \"quoted\" one\nline two\n\"\"\"")
            .tokenize()
            .unwrap();
        match &toks[0].0 {
            Tok::Str(parts) => match &parts[0] {
                LexStrPart::Text(t) => {
                    assert!(t.contains("line \"quoted\" one\nline two"), "{t}")
                }
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        }
    }
}
