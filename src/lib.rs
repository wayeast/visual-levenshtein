use unicode_segmentation::UnicodeSegmentation;

/// Instantiate a Levenshtein calculator.
///
/// A Levenshtein struct can generate several products:
/// 1) a Levenshtein distance score
/// ```
/// use visual_levenshtein::{levenshtein, Edit};
/// assert_eq!(3, levenshtein("kitten", "sitting").distance());
/// ```
/// 2) a string formatted according to a function argument representing the edits
///    that change string a -> string b:
/// ```
/// use visual_levenshtein::{levenshtein, Edit};
/// let encoder = |e: Edit| match e {
///     Edit::Equality(s) => format!("{}", s),
///     Edit::Deletion(s) => format!("[-{}-]", s),
///     Edit::Insertion(s) => format!("{{+{}+}}", s),
///     Edit::Substitution(o, d) => format!("[-{}-]{{+{}+}}", o, d),
/// };
/// let test = levenshtein("Saturday", "Sunday").encoded_edits(encoder);
/// let expected = "S[-at-]u[-r-]{+n+}day".to_string();
/// assert_eq!(expected, test);
/// ```
/// One can also access the grouped edits -- consecutive Transformations grouped by
/// like type:
/// ```
/// use visual_levenshtein::{levenshtein, Edit};
/// let test = levenshtein("kitten", "sitting").grouped_edits();
/// let expected = vec![
///     Edit::Substitution("k".to_string(), "s".to_string()),
///     Edit::Equality("itt".to_string()),
///     Edit::Substitution("e".to_string(), "i".to_string()),
///     Edit::Equality("n".to_string()),
///     Edit::Insertion("g".to_string()),
/// ];
/// assert_eq!(expected, test);
/// ```
/// Or the raw Transformations -- one character at a time:
/// ```
/// use visual_levenshtein::{levenshtein, Transformation};
/// let test = levenshtein("cats", "cup").raw_edits();
/// let expected = vec![
///     Transformation::Equality(0, "c"),
///     Transformation::Deletion(1, "a"),
///     Transformation::Substitution(2, "t", "u"),
///     Transformation::Substitution(3, "s", "p"),
/// ];
/// assert_eq!(expected, test);
/// ```
pub fn levenshtein<'a>(origin: &'a str, dest: &'a str) -> Levenshtein<'a> {
    Levenshtein::new(origin, dest)
}

pub fn levenshtein_words<'a>(origin: &'a str, dest: &'a str) -> Levenshtein<'a> {
    Levenshtein::new_words(origin, dest)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Transformation<'a> {
    Init(usize),
    Equality(usize, &'a str),
    Deletion(usize, &'a str),
    Insertion(usize, &'a str),
    Substitution(usize, &'a str, &'a str),
}

impl<'a> Transformation<'a> {
    fn cost(&'a self) -> usize {
        match self {
            Self::Init(c) => *c,
            Self::Equality(c, _) => *c,
            Self::Deletion(c, _) => *c,
            Self::Insertion(c, _) => *c,
            Self::Substitution(c, _, _) => *c,
        }
    }

    fn t(&'a self) -> usize {
        match self {
            Self::Init(_) => 0,
            Self::Equality(_, _) => 1,
            Self::Deletion(_, _) => 2,
            Self::Insertion(_, _) => 3,
            Self::Substitution(_, _, _) => 4,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Edit {
    Equality(String),
    Deletion(String),
    Insertion(String),
    Substitution(String, String),
}

#[derive(Debug)]
pub struct Levenshtein<'a> {
    x_dim: usize,
    y_dim: usize,
    origin: Vec<&'a str>,
    dest: Vec<&'a str>,
    matrix: Vec<Vec<Transformation<'a>>>,
}

impl<'a> Levenshtein<'a> {
    fn new(o: &'a str, d: &'a str) -> Self {
        let origin = UnicodeSegmentation::graphemes(o, true).collect::<Vec<&'a str>>();
        let dest = UnicodeSegmentation::graphemes(d, true).collect::<Vec<&'a str>>();
        let x_dim = origin.len() + 1;
        let y_dim = dest.len() + 1;
        let matrix = vec![vec![Transformation::Init(0); y_dim]; x_dim];

        Self {
            x_dim,
            y_dim,
            origin,
            dest,
            matrix,
        }
    }

    fn new_words(o: &'a str, d: &'a str) -> Self {
        let origin = UnicodeSegmentation::split_word_bounds(o).collect::<Vec<&'a str>>();
        let dest = UnicodeSegmentation::split_word_bounds(d).collect::<Vec<&'a str>>();
        let x_dim = origin.len() + 1;
        let y_dim = dest.len() + 1;
        let matrix = vec![vec![Transformation::Init(0); y_dim]; x_dim];

        Self {
            x_dim,
            y_dim,
            origin,
            dest,
            matrix,
        }
    }

    fn value_at(&self, x: usize, y: usize) -> Transformation<'a> {
        self.matrix[x][y].clone()
    }

    fn set_value(&mut self, x: usize, y: usize, val: Transformation<'a>) {
        self.matrix[x][y] = val;
    }

    fn initialize(&mut self) {
        self.set_value(0, 0, Transformation::Init(0));
        for x in 1..self.x_dim {
            self.set_value(x, 0, Transformation::Deletion(x, self.origin[x - 1]));
        }
        for y in 1..self.y_dim {
            self.set_value(0, y, Transformation::Insertion(y, self.dest[y - 1]));
        }
    }

    fn calculate_matrix(&mut self) {
        for x in 1..self.x_dim {
            for y in 1..self.y_dim {
                let deletion_cost = self.value_at(x - 1, y).cost() + 1;
                let deletion = Transformation::Deletion(deletion_cost, self.origin[x - 1]);
                let insertion_cost = self.value_at(x, y - 1).cost() + 1;
                let insertion = Transformation::Insertion(insertion_cost, self.dest[y - 1]);
                let sub_or_eq = t_delta(
                    self.value_at(x - 1, y - 1).cost(),
                    self.origin[x - 1],
                    self.dest[y - 1],
                );
                self.set_value(x, y, t_min_3(&deletion, &insertion, &sub_or_eq).clone());
            }
        }
    }

    pub fn distance(&mut self) -> usize {
        let x = self.origin.len();
        let y = self.dest.len();
        self.initialize();
        self.calculate_matrix();
        self.value_at(x, y).cost()
    }

    pub fn raw_edits(&mut self) -> Vec<Transformation<'a>> {
        let mut x = self.origin.len();
        let mut y = self.dest.len();
        self.initialize();
        self.calculate_matrix();
        let mut transformations: Vec<Transformation<'a>> = vec![];
        while x > 0 || y > 0 {
            let next = self.value_at(x, y);
            match next {
                Transformation::Insertion(_, _) => {
                    y -= 1;
                }
                Transformation::Deletion(_, _) => {
                    x -= 1;
                }
                Transformation::Equality(_, _) | Transformation::Substitution(_, _, _) => {
                    x -= 1;
                    y -= 1;
                }
                Transformation::Init(_) => {
                    unimplemented!("This should only be reached if x == 0 && y == 0!")
                }
            }
            transformations.push(next);
        }

        transformations.reverse();

        transformations
    }

    pub fn grouped_edits(&mut self) -> Vec<Edit> {
        let raw = self.raw_edits();
        let mut grouped: Vec<Edit> = vec![];
        let mut bin: Vec<&'a str> = vec![];
        let mut sub_dest_bin: Vec<&'a str> = vec![];
        let mut i: usize = 0;
        let mut current_t = raw[i].t();
        match raw[i] {
            Transformation::Equality(_, e) => bin.push(e),
            Transformation::Deletion(_, e) => bin.push(e),
            Transformation::Insertion(_, e) => bin.push(e),
            Transformation::Substitution(_, o, d) => {
                bin.push(o);
                sub_dest_bin.push(d);
            }
            Transformation::Init(_) => unimplemented!("This should never appear in raw edits!"),
        }
        i += 1;

        while i < raw.len() {
            while i < raw.len() && raw[i].t() == current_t {
                // push to bins
                match raw[i] {
                    Transformation::Equality(_, e) => bin.push(e),
                    Transformation::Deletion(_, e) => bin.push(e),
                    Transformation::Insertion(_, e) => bin.push(e),
                    Transformation::Substitution(_, o, d) => {
                        bin.push(o);
                        sub_dest_bin.push(d);
                    }
                    Transformation::Init(_) => {
                        unimplemented!("This should never appear in raw edits!")
                    }
                }
                i += 1;
            }
            // concatenate bins and push transform/s to grouped
            match current_t {
                1 => {
                    grouped.push(Edit::Equality(bin.join("")));
                }
                2 => {
                    grouped.push(Edit::Deletion(bin.join("")));
                }
                3 => {
                    grouped.push(Edit::Insertion(bin.join("")));
                }
                4 => {
                    grouped.push(Edit::Substitution(bin.join(""), sub_dest_bin.join("")));
                }
                _ => {
                    unimplemented!("This should never appear in raw edits!");
                }
            }
            current_t = if i < raw.len() { raw[i].t() } else { 0 };
            // clear bins
            bin = vec![];
            sub_dest_bin = vec![];
        }

        grouped
    }

    pub fn encoded_edits<F>(&mut self, encoder: F) -> String
    where
        F: Fn(Edit) -> String,
    {
        let grouped = self.grouped_edits();
        let components: Vec<String> = grouped.into_iter().map(|g| encoder(g)).collect();

        components.join("")
    }
}

fn t_min_3<'a, 'b>(
    insertion: &'b Transformation<'a>,
    deletion: &'b Transformation<'a>,
    sub_or_eq: &'b Transformation<'a>,
) -> &'b Transformation<'a> {
    let insertion_v_deletion = if insertion.cost() < deletion.cost() {
        insertion
    } else {
        deletion
    };
    if insertion_v_deletion.cost() < sub_or_eq.cost() {
        insertion_v_deletion
    } else {
        sub_or_eq
    }
}

fn t_delta<'a>(from_cost: usize, origin: &'a str, dest: &'a str) -> Transformation<'a> {
    if origin == dest {
        Transformation::Equality(from_cost, dest)
    } else {
        Transformation::Substitution(from_cost + 1, origin, dest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug() {
        //let mut c = levenshtein("kitten", "sitting");
        let mut c = levenshtein("one", "only");
        println!("New: {:?}", c);
        c.initialize();
        println!("Initialized: {:?}", c);
        c.calculate_matrix();
        println!("Calculated: {:?}", c);
        let transforms = c.raw_edits();
        println!("Edits: {:?}", transforms);
    }

    #[test]
    fn debug_words() {
        let mut c = levenshtein_words("one too many", "one too much, hey");
        println!("New: {:?}", c);
        c.initialize();
        println!("Initialized: {:?}", c);
        c.calculate_matrix();
        println!("Calculated: {:?}", c);
        let transforms = c.raw_edits();
        println!("Edits: {:?}", transforms);
    }

    #[test]
    fn matrix_initializes_correctly() {
        let mut c = levenshtein("ab", "ab");
        c.initialize();
        let expected: Vec<Vec<Transformation>> = vec![
            vec![
                Transformation::Init(0),
                Transformation::Insertion(1, "a"),
                Transformation::Insertion(2, "b"),
            ],
            vec![
                Transformation::Deletion(1, "a"),
                Transformation::Init(0),
                Transformation::Init(0),
            ],
            vec![
                Transformation::Deletion(2, "b"),
                Transformation::Init(0),
                Transformation::Init(0),
            ],
        ];
        assert_eq!(expected, c.matrix);
    }

    #[test]
    fn distance_checks() {
        assert_eq!(0, levenshtein("same", "same").distance());
        assert_eq!(1, levenshtein("same", "some").distance());
        assert_eq!(0, levenshtein("", "").distance());
        assert_eq!(3, levenshtein("kitten", "sitting").distance());
        assert_eq!(3, levenshtein("", "abc").distance());
        assert_eq!(3, levenshtein("abc", "").distance());
        assert_eq!(6, levenshtein("1234567", "7654321").distance());
        assert_eq!(4, levenshtein("11110000", "10101010").distance());
        assert_eq!(
            2,
            levenshtein("1010101010101010", "0101010101010101").distance()
        );
        assert_eq!(2, levenshtein("abcdefg", "gabcdef").distance());
    }

    #[test]
    fn raw_edit_checks() {
        // See examples at: https://en.wikipedia.org/wiki/Levenshtein_distance#Iterative_with_full_matrix
        let mut test = levenshtein("kitten", "sitting").raw_edits();
        let mut expected = vec![
            Transformation::Substitution(1, "k", "s"),
            Transformation::Equality(1, "i"),
            Transformation::Equality(1, "t"),
            Transformation::Equality(1, "t"),
            Transformation::Substitution(2, "e", "i"),
            Transformation::Equality(2, "n"),
            Transformation::Insertion(3, "g"),
        ];
        assert_eq!(expected, test);

        test = levenshtein("Saturday", "Sunday").raw_edits();
        expected = vec![
            Transformation::Equality(0, "S"),
            Transformation::Deletion(1, "a"),
            Transformation::Deletion(2, "t"),
            Transformation::Equality(2, "u"),
            Transformation::Substitution(3, "r", "n"),
            Transformation::Equality(3, "d"),
            Transformation::Equality(3, "a"),
            Transformation::Equality(3, "y"),
        ];
        assert_eq!(expected, test);

        test = levenshtein("", "").raw_edits();
        expected = vec![];
        assert_eq!(expected, test);
    }

    #[test]
    fn grouped_edit_checks() {
        let mut test = levenshtein("kitten", "sitting").grouped_edits();
        let mut expected = vec![
            Edit::Substitution("k".to_string(), "s".to_string()),
            Edit::Equality("itt".to_string()),
            Edit::Substitution("e".to_string(), "i".to_string()),
            Edit::Equality("n".to_string()),
            Edit::Insertion("g".to_string()),
        ];
        assert_eq!(expected, test);

        test = levenshtein("Saturday", "Sunday").grouped_edits();
        expected = vec![
            Edit::Equality("S".to_string()),
            Edit::Deletion("at".to_string()),
            Edit::Equality("u".to_string()),
            Edit::Substitution("r".to_string(), "n".to_string()),
            Edit::Equality("day".to_string()),
        ];
        assert_eq!(expected, test);

        test = levenshtein("abc", "def").grouped_edits();
        expected = vec![Edit::Substitution("abc".to_string(), "def".to_string())];
        assert_eq!(expected, test);
    }

    #[test]
    fn encoded_edits_checks() {
        let encoder = |e: Edit| match e {
            Edit::Equality(s) => format!("{}", s),
            Edit::Deletion(s) => format!("[-{}-]", s),
            Edit::Insertion(s) => format!("{{+{}+}}", s),
            Edit::Substitution(o, d) => format!("[-{}-]{{+{}+}}", o, d),
        };
        let test = levenshtein("cat", "cup").encoded_edits(encoder);
        let expected = "c[-at-]{+up+}".to_string();
        assert_eq!(expected, test);
    }

    #[test]
    fn t_min_3_always_prefers_lowest_cost() {
        let insertion = Transformation::Insertion(1, "");
        let deletion = Transformation::Deletion(2, "");
        let equality = Transformation::Equality(3, "");
        assert_eq!(&insertion, t_min_3(&insertion, &deletion, &equality));

        let insertion = Transformation::Insertion(1, "");
        let deletion = Transformation::Deletion(2, "");
        let substitution = Transformation::Substitution(3, "", "");
        assert_eq!(&insertion, t_min_3(&insertion, &deletion, &substitution));

        let insertion = Transformation::Insertion(3, "");
        let deletion = Transformation::Deletion(2, "");
        let equality = Transformation::Equality(1, "");
        assert_eq!(&equality, t_min_3(&insertion, &deletion, &equality));

        let insertion = Transformation::Insertion(3, "");
        let deletion = Transformation::Deletion(2, "");
        let substitution = Transformation::Substitution(1, "", "");
        assert_eq!(&substitution, t_min_3(&insertion, &deletion, &substitution));

        let insertion = Transformation::Insertion(2, "");
        let deletion = Transformation::Deletion(1, "");
        let equality = Transformation::Equality(3, "");
        assert_eq!(&deletion, t_min_3(&insertion, &deletion, &equality));

        let insertion = Transformation::Insertion(2, "");
        let deletion = Transformation::Deletion(1, "");
        let substitution = Transformation::Substitution(3, "", "");
        assert_eq!(&deletion, t_min_3(&insertion, &deletion, &substitution));
    }

    #[test]
    fn t_min_3_prefers_deletion_to_insertion() {
        let insertion = Transformation::Insertion(1, "");
        let deletion = Transformation::Deletion(1, "");
        let equality = Transformation::Equality(100, "");
        assert_eq!(&deletion, t_min_3(&insertion, &deletion, &equality));

        let insertion = Transformation::Insertion(1, "");
        let deletion = Transformation::Deletion(1, "");
        let substitution = Transformation::Substitution(100, "", "");
        assert_eq!(&deletion, t_min_3(&insertion, &deletion, &substitution));
    }

    #[test]
    fn t_min_3_prefers_substitution_or_equality_to_insertion_or_deletion() {
        let insertion = Transformation::Insertion(1, "");
        let deletion = Transformation::Deletion(100, "");
        let equality = Transformation::Equality(1, "");
        assert_eq!(&equality, t_min_3(&insertion, &deletion, &equality));

        let insertion = Transformation::Insertion(1, "");
        let deletion = Transformation::Deletion(100, "");
        let substitution = Transformation::Substitution(1, "", "");
        assert_eq!(&substitution, t_min_3(&insertion, &deletion, &substitution));

        let insertion = Transformation::Insertion(100, "");
        let deletion = Transformation::Deletion(1, "");
        let equality = Transformation::Equality(1, "");
        assert_eq!(&equality, t_min_3(&insertion, &deletion, &equality));

        let insertion = Transformation::Insertion(100, "");
        let deletion = Transformation::Deletion(1, "");
        let substitution = Transformation::Substitution(1, "", "");
        assert_eq!(&substitution, t_min_3(&insertion, &deletion, &substitution));
    }

    #[test]
    fn t_delta_checks() {
        assert_eq!(Transformation::Equality(0, "a"), t_delta(0, "a", "a"));
        assert_eq!(
            Transformation::Substitution(1, "a", "b"),
            t_delta(0, "a", "b")
        );
        assert_eq!(Transformation::Equality(0, "a̐"), t_delta(0, "a̐", "a̐"));
        assert_eq!(
            Transformation::Substitution(1, "a̐", "ö̲"),
            t_delta(0, "a̐", "ö̲")
        );
        assert_eq!(Transformation::Equality(0, "🇸🇹"), t_delta(0, "🇸🇹", "🇸🇹"));
        assert_eq!(
            Transformation::Substitution(1, "🇷🇺", "🇸🇹"),
            t_delta(0, "🇷🇺", "🇸🇹")
        );
    }
}
