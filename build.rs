use std::fmt::Write;
use std::{env, fs};

const FONT: &str = "assets/fonts/08-Schumacher-Clean.bdf";
// printable ASCII
const FIRST: u32 = 0x20;
const LAST: u32 = 0x7E;

fn main() {
    generate_font();
}

fn generate_font() {
    println!("cargo:rerun-if-changed={FONT}");
    let bdf = fs::read_to_string(FONT).expect("font file missing");

    let cell = field(&bdf, "FONTBOUNDINGBOX");
    let (width, height) = (cell[0], cell[1] as usize);
    assert!(width <= 8, "glyph rows must fit in one byte");

    let mut glyphs = vec![None; (LAST - FIRST + 1) as usize];
    for glyph in bdf.split("STARTCHAR").skip(1) {
        let code = field(glyph, "ENCODING")[0] as u32;
        if !(FIRST..=LAST).contains(&code) {
            continue;
        }
        // only full-cell glyphs, so bitmap rows map directly onto the cell
        assert_eq!(field(glyph, "BBX"), cell, "glyph {code} does not fill the cell");

        let rows: Vec<u8> = glyph
            .split_once("BITMAP")
            .expect("glyph without bitmap")
            .1
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .take_while(|line| *line != "ENDCHAR")
            .map(|line| u8::from_str_radix(line, 16).expect("bad bitmap row"))
            .collect();
        assert_eq!(rows.len(), height, "glyph {code} has the wrong height");
        glyphs[(code - FIRST) as usize] = Some(rows);
    }

    let mut out = format!(
        "pub const WIDTH: usize = {width};\n\
         pub const HEIGHT: usize = {height};\n\
         pub const FIRST: char = {:?};\n\
         static GLYPHS: [[u8; HEIGHT]; {}] = [\n",
        char::from_u32(FIRST).unwrap(),
        glyphs.len(),
    );
    for (index, rows) in glyphs.iter().enumerate() {
        let rows = rows.as_ref().unwrap_or_else(|| panic!("glyph {} missing", FIRST + index as u32));
        let rows: Vec<String> = rows.iter().map(|row| format!("{row:#04x}")).collect();
        writeln!(out, "    [{}],", rows.join(", ")).unwrap();
    }
    out.push_str("];\n");

    fs::write(format!("{}/font.rs", env::var("OUT_DIR").unwrap()), out).unwrap();
}

fn field(text: &str, name: &str) -> Vec<i32> {
    let line = text
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap_or_else(|| panic!("{name} missing"));
    line.split_whitespace().skip(1).map(|word| word.parse().unwrap()).collect()
}
