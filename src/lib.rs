extern crate unicode_segmentation;
extern crate unicode_width;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;


/// `String` with Unicode width 1. The text equivalent of a pixel.
#[derive(Clone)]
struct Char(String);

impl Char {
    pub fn new(s: &str) -> Result<Self, ()> {
        let width = s.width();

        if width != 1 {
            return Err(());
        }

        let c = Char(s.to_string());

        Ok(c)
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

#[derive(Clone)]
pub struct Stamp {
    data: Vec<Vec<Char>>,  // Row-major
    height: usize,
    width: usize,
}

impl Stamp {
    pub fn new(s: &str) -> Result<Self, ()> {
        Self::from_rectangle(&to_rectangle(s)?)
    }

    pub fn from_rectangle(s: &str) -> Result<Self, ()> {
        let rows: Vec<String> = s.split('\n').map(|s| s.to_string()).collect();

        let height = rows.len();

        // We must have at least one row.
        if height == 0 {
            return Err(());
        }

        let width = rows[0].width();

        // We must have at least one column.
        if width == 0 {
            return Err(());
        }

        // Each row must have the same width.
        if rows.iter().any(|s| s.width() != width) {
            return Err(());
        }

        let mut data: Vec<Vec<Char>> = vec![];

        for row in rows {
            let mut chars: Vec<Char> = vec![];

            for g in row.graphemes(true) {
                let c = Char::new(g)?;
                chars.push(c);
            }

            data.push(chars);
        }

        Ok(Stamp {
            data,
            height,
            width,
        })
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn rows(&self) -> Vec<String> {
        self.data
            .iter()
            .map(|chars| {
                let strings: Vec<String> = chars
                    .iter()
                    .map(|c| c.to_string())
                    .collect();
                strings.join("")
            })
            .collect()
    }

    pub fn render(&self) -> String {
        self.rows().join("\n")
    }

    pub fn layer(&self, other: &Stamp, col: usize, row: usize) -> Result<Stamp, ()> {
        if self.width() <= col || self.height() <= row {
            return Err(());
        }

        let mut stamp = self.clone();

        let max_col_index = std::cmp::min(col + other.width(), self.width());
        let max_row_index = std::cmp::min(row + other.height(), self.height());

        for r in row..max_row_index {
            for c in col..max_col_index {
                stamp.data[r][c] = other.data[r - row][c - col].clone();
            }
        }

        Ok(stamp)
    }
}

fn to_rectangle(s: &str) -> Result<String, ()> {
    if s.is_empty() {
        return Err(());
    }

    let rows: Vec<String> = s.split('\n').map(|s| s.to_string()).collect();

    let max_width = rows
        .iter()
        .map(|r| r.width())
        .max()
        .unwrap();

    let mut out = String::new();

    for (i, r) in rows.iter().enumerate() {
        out += &r.to_string();
        let w = r.width();
        let p = max_width - w;
        for _ in 0..p {
            out += " ";
        }
        if i != rows.len() - 1 {
            out += "\n";
        }
    }

    Ok(out)
}


#[cfg(test)]
mod tests {
    use super::{Stamp, to_rectangle};

    const VALID_STAMPS_LEN: usize = 12;
    type ValidStamps = [&'static str; VALID_STAMPS_LEN];
    const VALID_STAMPS: ValidStamps = [
        // ASCII only
        "a",
        "a\nb",
        "a\nb\nc",
        "aaa",
        "aaa\nbbb",
        "aaa\nbbb\nccc",

        // With combining characters
        "a̅",
        "a̅\nb̅",
        "a̅\nb̅\nc̅",
        "xa̅",
        "xa̅\nyb̅",
        "xa̅\nyb̅\nzc̅",
    ];

    const INVALID_STAMPS_LEN: usize = 20;
    type InvalidStamps = [&'static str; INVALID_STAMPS_LEN];
    const INVALID_STAMPS: InvalidStamps = [
        // Empty
        "",

        // Missing row
        "a\n",
        "\na",
        "\nab",
        "a\n\nb",
        "ab\n",

        // Unbalanced
        "a\nbc",
        "ab\nc",

        // Unbalanced row
        "a\nbcd",
        "abc\nd",
        "abc\nd",

        "a\nb\ncd",
        "a\nbc\nd",
        "ab\nc\nd",
        "abc\nd",
        "a\nbcd",

        // With combining characters
        "a̅\nbc",
        "ab\nc̅",
        "a̅b\ncde",
        "abc\nc̅d",
    ];

    #[test]
    fn test_from_rectangle_valid() {
        for s in &VALID_STAMPS {
            let st = Stamp::from_rectangle(s);
            assert!(st.is_ok(), "Should be a valid stamp: {:?}", s);
        }
    }

    #[test]
    fn test_from_rectangle_invalid() {
        for s in &INVALID_STAMPS {
            let st = Stamp::from_rectangle(s);
            assert!(st.is_err(), "Should not be a valid stamp: {:?}", s);
        }
    }

    #[test]
    fn test_render() {
        for s in &VALID_STAMPS {
            let st = Stamp::from_rectangle(s).ok().unwrap();

            let out = st.render();

            assert_eq!(&out, s, "{:?} should equal {:?}", out, s);
        }
    }

    #[test]
    fn test_layer() {
        let s1 = "oooooooooo\noooooooooo\noooooooooo\noooooooooo";
        let s2 = "xxx\nxxx";

        let st1 = Stamp::from_rectangle(s1).ok().unwrap();
        let st2 = Stamp::from_rectangle(s2).ok().unwrap();

        let out_0_0 = st1.layer(&st2, 0, 0).ok().unwrap().render();
        assert_eq!(&out_0_0, "xxxooooooo\nxxxooooooo\noooooooooo\noooooooooo");

        let out_3_1 = st1.layer(&st2, 3, 1).ok().unwrap().render();
        assert_eq!(&out_3_1, "oooooooooo\noooxxxoooo\noooxxxoooo\noooooooooo");

        let out_8_2 = st1.layer(&st2, 8, 2).ok().unwrap().render();
        assert_eq!(&out_8_2, "oooooooooo\noooooooooo\nooooooooxx\nooooooooxx");
    }

    #[test]
    fn test_to_rectangle() {
        assert!(to_rectangle("").is_err());

        assert_eq!(to_rectangle("a").ok().unwrap(), "a");
        assert_eq!(to_rectangle("a\n").ok().unwrap(), "a\n ");
        assert_eq!(to_rectangle("\na").ok().unwrap(), " \na");
        assert_eq!(to_rectangle("\nab").ok().unwrap(), "  \nab");
        assert_eq!(to_rectangle("a\nb").ok().unwrap(), "a\nb");
        assert_eq!(to_rectangle("ab\n").ok().unwrap(), "ab\n  ");
        assert_eq!(to_rectangle("\na\nbc").ok().unwrap(), "  \na \nbc");
        assert_eq!(to_rectangle("aaa\nb\ncc").ok().unwrap(), "aaa\nb  \ncc ");
    }
}
