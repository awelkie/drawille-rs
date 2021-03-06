use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::cmp;
use std::default::Default;
use std::fmt::{self, Formatter};
use std::ops::{Index, IndexMut};

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
struct ColorPair(Color, Color);

impl fmt::Display for ColorPair {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // TODO: add Windows support if needed
        let ColorPair(first, second) = *self;
        let finit = "\x1b[0;";
        let fend = first as u32;
        let f = format!("{}4{}m", finit, fend);
        let sinit = "\x1b[";
        let send = second as u32;
        let s = format!("{}3{}m", sinit, send);
        try!(write!(fmt, "{}{}", f, s));
        Ok(())
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
enum Pixel {
    Char(ColorPair, char),
    Pair(ColorPair),
}

impl Default for Pixel {
    fn default() -> Pixel {
        Pixel::Char(ColorPair(Color::Black, Color::Black), ' ')
    }
}

impl fmt::Display for Pixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Pixel::Char(cp, a) => try!(write!(f, "{}{}", cp, a)),
            Pixel::Pair(a) => try!(write!(f, "{}▄", a)),
        }
        Ok(())
    }
}

impl Index<usize> for Pixel {
    type Output = Color;

    fn index<'a>(&'a self, index: usize) -> &'a Color {
        let cp = match *self {
            Pixel::Pair(ref cp) => cp,
            _ => panic!("indexing a text pixel"),
        };
        let ColorPair(ref c1, ref c2) = *cp;
        match index {
            0 => c1,
            1 => c2,
            _ => panic!("ColorPair index out of bounds"),
        }
    }
}

impl IndexMut<usize> for Pixel {
    fn index_mut<'a>(&'a mut self, index: usize) -> &'a mut Color {
        let cp = match *self {
            Pixel::Pair(ref mut cp) => cp,
            _ => panic!("indexing a text pixel"),
        };
        let ColorPair(ref mut c1, ref mut c2) = *cp;
        match index {
            0 => c1,
            1 => c2,
            _ => panic!("ColorPair index out of bounds"),
        }
    }
}

impl Pixel {
    fn index(&self, index: usize) -> Color {
        let cp = match *self {
            Pixel::Pair(cp) => cp,
            _ => panic!("indexing a text pixel"),
        };
        let ColorPair(c1, c2) = cp;
        match index {
            0 => c1,
            1 => c2,
            _ => panic!("ColorPair index out of bounds"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas {
    blocks: HashMap<(usize, usize), Pixel>,
    width:  usize,
    height: usize,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Canvas {
        Canvas {
            blocks: HashMap::new(),
            width: width / 2,
            height: height / 4,
        }
    }

    pub fn clear(&mut self) {
        self.blocks.clear();
    }

    pub fn text<S: AsRef<str>>(&mut self, x: usize, y: usize, fg: Color, bg: Color, s: S) {
        let (row, col) = (x, y / 2);
        for (i, c) in s.as_ref().chars().enumerate() {
            match self.blocks.entry((row + i, col)) {
                Entry::Occupied(e) => *e.into_mut() = Pixel::Char(ColorPair(bg, fg), c),
                Entry::Vacant(e) => { e.insert(Pixel::Char(ColorPair(bg, fg), c)); },
            }
        }
    }

    pub fn set(&mut self, x: usize, y: usize, c: Color) {
        let (row, col) = (x, y / 2);
        let mut block = match self.blocks.entry((row, col)) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(Default::default()),
        };
        match block {
            ref mut a @ &mut Pixel::Char(_, _) => **a = Pixel::Pair(ColorPair(Color::Black, Color::Black)),
            _ => {},
        }

        block[y % 2] = c;
    }

    pub fn unset(&mut self, x: usize, y: usize) {
        let (row, col) = (x, y / 2);
        let mut block = match self.blocks.entry((row, col)) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(Default::default()),
        };
        block[y % 2] = Color::Black;
    }

    pub fn get(&self, x: usize, y: usize) -> Color {
        let (col, row) = (x, y / 2);
        let col = self.blocks.get(&(row, col));

        match col {
            None => Color::Black,
            Some(c) => c.index(y % 2),
        }
    }

    pub fn rows(&self) -> Vec<String> {
        let maxrow = cmp::max(self.width, self.blocks.keys().map(|&(x, _)| x).max().unwrap_or(0));
        let maxcol = cmp::max(self.height, self.blocks.keys().map(|&(_, y)| y).max().unwrap_or(0));

        let mut result = vec![];
        for y in (0..maxcol + 1) {
            let mut row = String::new();
            for x in (0..maxrow + 1) {
                let col = *self.blocks.get(&(x, y)).unwrap_or(&Default::default());
                row.push_str(&format!("{}", col));
            }
            result.push(format!("{}\x1b[0m", row));
        }
        result
    }

    pub fn frame(&self) -> String {
        self.rows().connect("\n")
    }

    pub fn line_vec(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> Vec<(usize, usize)> {
        let xdiff = cmp::max(x1, x2) - cmp::min(x1, x2);
        let ydiff = cmp::max(y1, y2) - cmp::min(y1, y2);
        let xdir = if x1 <= x2 { 1 } else { -1 };
        let ydir = if y1 <= y2 { 1 } else { -1 };

        let r = cmp::max(xdiff, ydiff);

        let mut result = vec![];
        for i in (0..r + 1) {
            let mut x = x1 as isize;
            let mut y = y1 as isize;

            if ydiff != 0 {
                y += ((i * ydiff) / r) as isize * ydir;
            }
            if xdiff != 0 {
                x += ((i * xdiff) / r) as isize * xdir;
            }

            result.push((x as usize, y as usize));
        }
        result
    }

    pub fn line(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, c: Color) {
        for &(x, y) in self.line_vec(x1, y1, x2, y2).iter() {
            self.set(x, y, c);
        }
    }
}
