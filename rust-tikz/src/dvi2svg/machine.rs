use super::svgmachine::SVGMachine;
use dvi2html::tfm::FontDataHelper;
use dvi::{FontDef, Instruction};
use std::char;

type PositionUnit = f64; //TODO
#[derive(Copy, Clone, Debug)]
pub struct Position {
    h: PositionUnit,
    v: PositionUnit,
    right: [i32; 2],
    down: [i32; 2],
}

impl Position {
    pub fn empty() -> Position {
        Position {
            h: 0 as PositionUnit,
            v: 0 as PositionUnit,
            right: [0, 0],
            down: [0, 0],
        }
    }

    pub fn move_right(&mut self, distance: PositionUnit) {
        self.h += distance;
    }

    pub fn move_down(&mut self, distance: PositionUnit) {
        self.v += distance;
    }

    pub fn h(&self) -> PositionUnit {
        self.h
    }

    pub fn v(&self) -> PositionUnit {
        self.v
    }

    pub fn change_right(&mut self, o: Option<i32>, first: bool) {
        let index = if first { 0 } else { 1 };
        if let Some(v) = o {
            self.right[index] = v;
        }
        self.move_right(self.right[index] as f64);
    }

    pub fn change_down(&mut self, o: Option<i32>, first: bool) {
        let index = if first { 0 } else { 1 };
        if let Some(v) = o {
            self.down[index] = v;
        }
        self.move_down(self.down[index] as f64);
    }
}

#[allow(unused)]
#[derive(Clone)]
pub struct PreambleData {
    pub format: u8,
    pub numerator: u32,
    pub denominator: u32,
    pub magnification: u32,
    pub comment: String,
}

pub type SpecialHandler = Box<dyn Fn(&mut SVGMachine, &str) -> bool>;
pub trait Machine {
    /// Returns the resulting string (e.g., for HTML or SVG conversion).
    fn get_content(&self) -> String;
    fn put_text(&mut self, text: Vec<u32>, font_helper: &FontDataHelper) -> f64;
    fn put_rule(&mut self, a: i32, b: i32);
    fn begin_page(&mut self, arr: [i32; 10], p: i32);
    fn end_page(&mut self);
    fn push_position(&mut self);
    fn pop_position(&mut self);
    fn get_position(&mut self) -> &mut Position;
    fn set_font(&mut self, index: u32);
    fn add_font(&mut self, font: FontDef);
    fn set_preamble_data(&mut self, data: PreambleData);
    fn handle_special(&mut self, special_handlers: &[SpecialHandler], comment: &str);
    fn set_nb_pages(&mut self, nb_pages: u16);
}

pub trait Executor: Machine {
    fn execute(
        &mut self,
        instruction: &Instruction,
        font_helper: &FontDataHelper,
        special_handlers: &[SpecialHandler],
    ) -> Result<(), String> {
        //TODO: dereferences to borrows in Machine methods?
        match instruction {
            Instruction::Set(u) => {
                let oc = char::from_u32(*u); //TODO: is this needed here?
                match oc {
                    Some(_c) => {
                        let width = self.put_text(vec![*u], font_helper);
                        self.get_position().move_right(width);
                    }
                    None => return Err(format!("Invalid char bytes: {}", u)),
                }
            }
            Instruction::Put(u) => {
                self.put_text(vec![*u], font_helper); // TODO: see Set
            }
            Instruction::SetRule(a, b) => {
                self.put_rule(*a, *b);
                self.get_position().move_right(*b as f64);
            }
            Instruction::PutRule(a, b) => {
                self.put_rule(*a, *b);
            }
            Instruction::Nop => (),
            Instruction::Bop(arr, p) => self.begin_page(*arr, *p),
            Instruction::Eop => self.end_page(),
            Instruction::Push => self.push_position(),
            Instruction::Pop => self.pop_position(),
            Instruction::Right(d) => self.get_position().move_right(*d as f64),
            Instruction::W(o) => self.get_position().change_right(*o, true),
            Instruction::X(o) => self.get_position().change_right(*o, false),
            Instruction::Down(d) => self.get_position().move_down(*d as f64),
            Instruction::Y(o) => self.get_position().change_down(*o, true),
            Instruction::Z(o) => self.get_position().change_down(*o, false),
            Instruction::Font(f) => self.set_font(*f), //TODO: Xxx
            Instruction::Xxx(vec) => {
                self.handle_special(&special_handlers, std::str::from_utf8(&vec).unwrap())
            }
            Instruction::FontDef(def) => self.add_font(def.clone()),
            Instruction::Pre {
                format,
                numerator,
                denominator,
                magnification,
                comment,
            } => {
                //TODO: error if numerator <= 0 or denominator <= 0 or format != 2
                self.set_preamble_data(PreambleData {
                    format: *format,
                    numerator: *numerator,
                    denominator: *denominator,
                    magnification: *magnification,
                    comment: std::str::from_utf8(&comment).unwrap().to_string(), //TODO, handle error
                });
            }
            Instruction::Post {
                final_bop_pointer: _,
                numerator: _,
                denominator: _,
                magnification: _,
                tallest_height: _,
                widest_width: _,
                max_stack_depth: _,
                total_no_pages,
            } => self.set_nb_pages(*total_no_pages), //TODO?
            Instruction::PostPost {
                post_pointer: _,
                ident: _,
                two_two_three: _,
            } => (), //TODO?
        }
        //Always run a random special, so the specials can see that other instructions are happening
        match instruction {
            Instruction::Xxx(_) => (),
            _ => self.handle_special(&special_handlers, "somethingrandom"),
        }
        Ok(())
    }
}
