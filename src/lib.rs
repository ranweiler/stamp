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


#[cfg(test)]
mod tests {
    use super::Stamp;

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
    fn test_new_valid() {
        for s in &VALID_STAMPS {
            let st = Stamp::new(s);
            assert!(st.is_ok(), "Should be a valid stamp: {:?}", s);
        }
    }

    #[test]
    fn test_new_invalid() {
        for s in &INVALID_STAMPS {
            let st = Stamp::new(s);
            assert!(st.is_err(), "Should not be a valid stamp: {:?}", s);
        }
    }

    #[test]
    fn test_render() {
        for s in &VALID_STAMPS {
            let st = Stamp::new(s).ok().unwrap();

            let out = st.render();

            assert_eq!(&out, s, "{:?} should equal {:?}", out, s);
        }
    }

    #[test]
    fn test_layer() {
        let s1 = "oooooooooo\noooooooooo\noooooooooo\noooooooooo";
        let s2 = "xxx\nxxx";

        let st1 = Stamp::new(s1).ok().unwrap();
        let st2 = Stamp::new(s2).ok().unwrap();

        let out_0_0 = st1.layer(&st2, 0, 0).ok().unwrap().render();
        assert_eq!(&out_0_0, "xxxooooooo\nxxxooooooo\noooooooooo\noooooooooo");

        let out_3_1 = st1.layer(&st2, 3, 1).ok().unwrap().render();
        assert_eq!(&out_3_1, "oooooooooo\noooxxxoooo\noooxxxoooo\noooooooooo");

        let out_8_2 = st1.layer(&st2, 8, 2).ok().unwrap().render();
        assert_eq!(&out_8_2, "oooooooooo\noooooooooo\nooooooooxx\nooooooooxx");
    }
}
