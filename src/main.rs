use analyzer::{analyze, tokenize};

fn main() {
    let content: String = String::from("VAR A,K:ARRAY[2:10,10:40] OF BYTE, D17,E7 : WORD;");

    match tokenize(content.clone()) {
        Ok(tokens) => match analyze(tokens) {
            Ok(()) => {
                println!(
                    "String `{}` is a valid Turbo Pascal var declaration",
                    content
                );
            }
            Err(e) => {
                println!("{}", e)
            }
        },
        Err(e) => println!("{}", e),
    }
}
