use dvi2html::tfm;
use machine::{Executor, Machine};
use utils::parse_dvi;

pub(crate) mod svgmachine;
pub(crate) mod machine;
pub(crate) mod utils;

pub fn dvi2svg(input: &[u8]) -> Result<String, String> {
    let font_helper = tfm::FontDataHelper::init().unwrap();
    let mut machine = svgmachine::SVGMachine::new();
    let instructions = parse_dvi(input);
    let special_handlers: Vec<machine::SpecialHandler> = vec![
        Box::new(svgmachine::special_html_svg),
        Box::new(svgmachine::special_html_color),
        Box::new(svgmachine::special_html_papersize),
    ];
    for ins in instructions.iter() {
        let _ = machine.execute(ins, &font_helper, &special_handlers);
    }

    Ok(machine.get_content())
}