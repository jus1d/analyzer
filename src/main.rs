use analyzer::tokenize;

fn main() {
    let content: String = String::from("VAR A,K:ARRAY[2:10,10:40] OF BYTE, D17,E7 : WORD;");

    match tokenize(content) {
        Ok(tokens) => {
            println!("Parsed {} tokens", tokens.len());

            for tok in tokens {
                println!("{:?}", tok)
            }
        }
        Err(e) => println!("{}", e),
    }
}
