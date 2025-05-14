use super::machine::{Executor, Machine, Position, PreambleData, SpecialHandler};
use super::utils::tex_color_to_hex;
use crate::tfm::FontDataHelper;
use dvi::FontDef;
use std::char;
use std::collections::HashMap;

/// An implementation of `Machine` that generates an SVG as output.
#[derive(Debug)]
pub struct SVGMachine {
    content: String,
    color: String,
    color_stack: Vec<String>,
    points_per_dvi_unit: Option<f64>,
    svg_depth: u8,
    paperwidth: Option<f64>,
    paperheight: Option<f64>,

    position: Position,
    position_stack: Vec<Position>,
    font: Option<FontDef>,
    fonts: HashMap<u32, FontDef>,

    nb_pages: u16,

    svg_buffer: String,
}

impl SVGMachine {
    pub fn new() -> SVGMachine {
        SVGMachine {
            content: "".to_string(),
            color: "black".to_string(),
            color_stack: Vec::new(),
            points_per_dvi_unit: None,
            // We will wrap the entire document in an SVG tag, so we are always in an svg.
            svg_depth: 1,
            paperwidth: None,
            paperheight: None,
            position: Position::empty(),
            position_stack: Vec::new(),
            font: None,
            fonts: HashMap::new(),
            nb_pages: 0,
            svg_buffer: "".to_string(),
        }
    }
}

impl Machine for SVGMachine {
    fn get_content(&self) -> String {
        let width = self.paperwidth.unwrap_or(100.0).to_string();
        let height = self.paperheight.unwrap_or(100.0).to_string();
        format!(
            r#"<svg  version="1.1" xmlns="http://www.w3.org/2000/svg"
                width="{width}pt" height="{height}pt"
                viewBox="-72 -72 {width} {height}">
                <style>
                   text[font-family*="cmmi"] {{
                        font-family: "New Computer Modern Math";
                        font-style: italic;
                    }}
                </style>
                {}
</svg>"#,
            self.content
        )
    }
    fn set_nb_pages(&mut self, nb_pages: u16) {
        self.nb_pages = nb_pages;
    }
    fn get_position(&mut self) -> &mut Position {
        &mut self.position
    }
    fn put_text(&mut self, buffer: Vec<u32>, font_helper: &FontDataHelper) -> f64 {
        let mut text_width = 0;
        let mut text_height = 0;
        let mut text_depth = 0;

        let mut html_text = "".to_string();

        let font = self.font.as_ref().unwrap(); //TODO

        // TODO: better error handling
        let font_name = std::str::from_utf8(&font.filename).unwrap();
        let font_data = font_helper.get(font_name.to_string()).unwrap_or_else(|| {
            eprintln!("Using fallback cmb10 for {}", font_name);
            font_helper.get("cmb10".to_string()).unwrap()
        }); // Fallback if font not found

        for &c in buffer.iter() {
            let mut metrics_option = font_data.characters.get(&c);
            if metrics_option.is_none() {
                //TODO: Handle this better. Error only happens for c === 127
                eprintln!("Could not find font metric for {}", c);
                metrics_option = font_data.characters.get(&126);
            }
            if let Some(metrics) = metrics_option {
                text_width += metrics.width;
                text_height = std::cmp::max(text_height, metrics.height);
                text_depth = std::cmp::max(text_depth, metrics.depth);

                println!("Converting {c} to character {buffer:?}");
                // This is ridiculous.
                if c <= 9 {
                    html_text.push_str(&format!("&#{};", 161 + c));
                } else if c >= 10 && c <= 19 {
                    html_text.push_str(&format!("&#{};", 173 + c - 10));
                } else if c == 20 {
                    html_text.push_str("&#8729;"); // O RLLY?!
                } else if c >= 21 && c <= 32 {
                    html_text.push_str(&format!("&#{};", 184 + c - 21));
                } else if c == 127 {
                    html_text.push_str("&#196;");
                } else {
                    html_text.push_str(&char::from_u32(c).unwrap().to_string());
                    //TODO?
                }
            }
        }

        // tfm is based on 1/2^16 pt units, rather than dviunit which is 10^−7 meters
        let dvi_units_per_font_unit =
            (font_data.design_size as f64) / 1_048_576.0 * 65536.0 / 1_048_576.0;
        let points_per_dvi_unit = self.points_per_dvi_unit.unwrap(); //TODO

        // TODO: remove unused
        let _top = ((self.position.v() as f64) - (text_height as f64) * dvi_units_per_font_unit)
            * points_per_dvi_unit;
        let left = (self.position.h() as f64) * points_per_dvi_unit;

        let _width = (text_width as f64) * points_per_dvi_unit * dvi_units_per_font_unit;
        //let height = (text_height as f64) * points_per_dvi_unit * dvi_units_per_font_unit;
        let _depth = (text_depth as f64) * points_per_dvi_unit * dvi_units_per_font_unit;
        //let top = (self.position.v() as f64) * points_per_dvi_unit;

        let fontsize = ((font_data.design_size as f64) / 1_048_576.0) * (font.scale_factor as f64)
            / (font.design_size as f64);

        let color = &self.color;
        let bottom = (self.position.v() as f64) * points_per_dvi_unit;
        // No 'pt' on fontsize since those units are potentially scaled
        self.content.push_str(&format!(
                r#"
                <text y="{}" x="{}" font-family="{}" fill="{color}" font-size="{}">{}</text>"#,
                bottom, left, font_name, fontsize, html_text
            ));

        (text_width as f64) * dvi_units_per_font_unit * (font.scale_factor as f64)
            / (font.design_size as f64)
    }

    fn put_rule(&mut self, ai: i32, bi: i32) {
        let points_per_dvi_unit = self.points_per_dvi_unit.unwrap(); //TODO

        let a = (ai as f64) * points_per_dvi_unit;
        let b = (bi as f64) * points_per_dvi_unit;
        let left = self.position.h() * points_per_dvi_unit;
        let bottom = self.position.v() * points_per_dvi_unit;
        let top = bottom - a;

        self.content.push_str(&format!(
            r#"
        <rect x="{left}" y="{top}" width="{b}" height="{a}" fill="{}" stroke="none" />
        "#,
            self.color
        ));
    }
    fn begin_page(&mut self, _arr: [i32; 10], _p: i32) {
        self.position_stack.clear();
        //self.position = Position::empty(); //TODO: Optional
    }
    fn end_page(&mut self) {
        //TODO check if position stack is empty
    }
    fn push_position(&mut self) {
        self.position_stack.push(self.position.clone());
    }
    fn pop_position(&mut self) {
        self.position = self.position_stack.pop().unwrap(); //TODO?
    }
    fn set_font(&mut self, index: u32) {
        self.font = self.fonts.get(&index).cloned();
    }
    fn add_font(&mut self, font: FontDef) {
        self.fonts.insert(font.number, font);
    }
    fn set_preamble_data(&mut self, data: PreambleData) {
        let magnification = data.magnification as f64;
        let numerator = data.numerator as f64;
        let denominator = data.denominator as f64;
        let dvi_unit = (magnification * numerator) / (1000.0 * denominator);

        let resolution = 300.0; // ppi
        let _tfm_conv = (25_400_000.0 / numerator) * (denominator / 473_628_672.0) / 16.0;
        let _conv = (numerator / 254_000.0) * (resolution / denominator) * (magnification / 1000.0);

        self.points_per_dvi_unit = Some(dvi_unit * 72.27 / 100_000.0 / 2.54);
    }
    fn handle_special(&mut self, special_handlers: &[SpecialHandler], command: &str) {
        for special in special_handlers.iter() {
            if special(self, command) {
                break;
            }
        }
    }
}

impl Executor for SVGMachine {}

//Specials -> maybe PopColor etc to Machine trait
impl SVGMachine {
    fn special_color(&mut self, command: &str) -> bool {
        if command.starts_with("color pop") {
            self.color = self.color_stack.pop().unwrap(); //TODO
            return true;
        } else if command.starts_with("color push ") {
            let color = tex_color_to_hex(command.split_at("color push ".len()).1);
            self.color_stack.push(color.to_string());
            self.color = color;
            return true;
        }
        false
    }

    fn special_papersize(&mut self, command: &str) -> bool {
        let pattern = "papersize=";
        if command.starts_with(pattern) {
            let sizes = command
                .split_at(pattern.len())
                .1
                .split(',')
                .collect::<Vec<_>>();
            //TODO: error if sizes is not of len 2
            //Error if first or second element doesn't end with 'pt'

            let width = Some(
                sizes[0]
                    .split_at(sizes[0].len() - 2)
                    .0
                    .parse::<f64>()
                    .unwrap(),
            ); //TODO
            let height = Some(
                sizes[1]
                    .split_at(sizes[1].len() - 2)
                    .0
                    .parse::<f64>()
                    .unwrap(),
            ); //TODO
            self.paperwidth = width;
            self.paperheight = height;
        }
        false
    }

    fn append_svg(&mut self, s: &str) {
        self.svg_buffer.push_str(s);
    } //TODO: go to all specials for every special handler

    fn put_svg(&mut self) {
        let points_per_dvi_unit = self.points_per_dvi_unit.unwrap();
        let left = self.position.h() * points_per_dvi_unit;
        let top = self.position.v() * points_per_dvi_unit;

        self.svg_depth += self.svg_buffer.matches("<svg>").count() as u8;
        self.svg_depth -= self.svg_buffer.matches("</svg>").count() as u8;

        let mut result_svg = self.svg_buffer.clone();
        result_svg = result_svg.replace(r#"{?x}"#, &format!("{}", left));
        result_svg = result_svg.replace(r#"{?y}"#, &format!("{}", top));
        result_svg = result_svg.replace(r#"{?nl}"#, "\n");
        // Tikz will leave <svg beginpicture> and </svg endpicture> tags in the svg.
        // Since we wrap the entire document in an svg tag, we don't need them.
        result_svg = result_svg.replace(r#"<svg beginpicture>"#, "");
        result_svg = result_svg.replace(r#"</svg endpicture>"#, "");

        self.content.push_str(&result_svg);
        self.svg_buffer = "".to_string();
    }

    fn special_svg(&mut self, command: &str) -> bool {
        let pattern = "dvisvgm:raw ";
        if command.starts_with(pattern) {
            let svg = command.split_at(pattern.len()).1;
            self.append_svg(svg);
            return true;
        } else if !self.svg_buffer.is_empty() {
            self.put_svg();
        }
        false
    }
}

pub fn special_html_color(m: &mut SVGMachine, command: &str) -> bool {
    m.special_color(command)
}

pub fn special_html_papersize(m: &mut SVGMachine, command: &str) -> bool {
    m.special_papersize(command)
}

pub fn special_html_svg(m: &mut SVGMachine, command: &str) -> bool {
    m.special_svg(command)
}
