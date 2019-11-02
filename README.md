# visual-levenshtein

A Levenshtein distance calculator that produces not just distances but also edit mappings.

#### Find the classic Levenshtein distance between two strings:

```rust
assert_eq!(3, levenshtein("kitten", "sitting").distance());
```

#### Show the edits that transform one string into another:
This function takes an encoding function as argument, making
the way edits are rendered infinitely configurable and customizable!

```rust
// Encode edits in `wdiff`-like notation
let encoder = |e: Edit| match e {
    Edit::Equality(s) => format!("{}", s),
    Edit::Deletion(s) => format!("[-{}-]", s),
    Edit::Insertion(s) => format!("{{+{}+}}", s),
    Edit::Substitution(o, d) => format!("[-{}-]{{+{}+}}", o, d),
};
let test = levenshtein("Saturday", "Sunday").encoded_edits(encoder);
let expected = "S[-at-]u[-r-]{+n+}day".to_string();
assert_eq!(expected, test);
```

One can also access lower-level data structs, like grouped edits -- concatenated sequences
of like transformations:

```rust
let test = levenshtein("kitten", "sitting").grouped_edits();
let expected = vec![
    Edit::Substitution("k".to_string(), "s".to_string()),
    Edit::Equality("itt".to_string()),
    Edit::Substitution("e".to_string(), "i".to_string()),
    Edit::Equality("n".to_string()),
    Edit::Insertion("g".to_string()),
];
assert_eq!(expected, test);
```

Or raw edits -- single-character transformations:

```rust
let test = levenshtein("cats", "cup").raw_edits();
let expected = vec![
    Transformation::Equality(0, "c"),
    Transformation::Deletion(1, "a"),
    Transformation::Substitution(2, "t", "u"),
    Transformation::Substitution(3, "s", "p"),
];
assert_eq!(expected, test);
```
