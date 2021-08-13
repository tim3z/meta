use pulldown_cmark::{html, Parser};

fn main() {
    let filename = std::env::args().skip(1).next().expect("Need input file");
    let contents = std::fs::read_to_string(filename)
        .expect("Something went wrong reading the file");

    let parser = Parser::new(&contents).into_offset_iter();

    let parser = parser.map(|(e, r)| {
        dbg!(&e,r);
        e
    });

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    eprintln!("{}", html_output);
}
