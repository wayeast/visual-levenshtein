use visual_levenshtein::{levenshtein_words, Edit};

fn main() {
    let encoder = |e: Edit| match e {
        Edit::Equality(s) => format!("{}", s),
        Edit::Deletion(s) => format!("[-{}-]", s),
        Edit::Insertion(s) => format!("{{+{}+}}", s),
        Edit::Substitution(o, d) => format!("[-{}-]{{+{}+}}", o, d),
    };
    let examples: Vec<(&str, &str)> = vec![
        (
            "One fine day in spring, ...",
            "One fine man said May day ...",
        ),
        (
            "One fine day in spring, ...",
            "One fine man said mayday ...",
        ),
        (
            "One fine day in spring, ...",
            "One fine man said Spring ...",
        ),
    ];
    for example in examples {
        let encoded = levenshtein_words(example.0, example.1).encoded_edits(encoder);
        println!("'{}' -> '{}': '{}'", example.0, example.1, encoded);
    }
}
