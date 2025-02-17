use figlet_rs::FIGfont;

pub fn print_banner() {
    let slant_font_data = include_str!("../resources/slant.flf");
    let slant_font = FIGfont::from_content(slant_font_data).unwrap();
    let figure = slant_font.convert("Banner");
    print!("{}", figure.unwrap());
    println!("Bugs or issues: https://github.com/sogladev/rs_manifest_patcher/");
    println!("\n{}", "-".repeat(100));
}
