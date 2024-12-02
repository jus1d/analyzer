use analyzer::{analyze, tokenize};
use iced::widget::{button, column, row, text, text_input, Column};

#[derive(Default)]
struct App {
    source: String,
    success: String,
    error: String,
    identifiers: String,

    before_error: String,
    error_token: String,
    after_error: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    SourceChanged(String),
    Process,
}

impl App {
    pub fn view(&self) -> Column<Message> {
        let title = text("Turbo Pascal VAR analyzer").size(24);

        let input = text_input("Source", &self.source)
            .id("source")
            .on_input(Message::SourceChanged)
            .on_submit(Message::Process)
            .size(20);

        let process_button = button("Process").on_press(Message::Process);

        let error_message = text(&self.error).color([1.0, 0.0, 0.0]).size(20);
        let success_message = text(&self.success).color([0.4, 0.7, 0.0]).size(20);

        let idents_table = text(&self.identifiers).size(20);

        let report = if self.source.is_empty() || self.error.is_empty() {
            row!()
        } else {
            row![column![
                text("Error sample:").size(20),
                row!(
                    text(&self.before_error).size(20),
                    text(&self.error_token).color([1.0, 0.0, 0.0]).size(20),
                    text(&self.after_error).size(20),
                )
            ]]
        };

        const SPACING: u16 = 20;
        if !self.error.is_empty() {
            return Column::new()
                .padding(20)
                .spacing(SPACING)
                .push(title)
                .push(input)
                .push(process_button)
                .push(error_message)
                .push(report)
                .into();
        } else {
            return Column::new()
                // .align_x(Center)
                .padding(20)
                .spacing(SPACING)
                .push(title)
                .push(input)
                .push(process_button)
                .push(success_message)
                .push(idents_table);
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SourceChanged(source) => {
                self.source = source;
                self.clear();
            }
            Message::Process => match tokenize(self.source.clone()) {
                Ok(tokens) => match analyze(tokens) {
                    Ok(identifiers) => {
                        self.success = format!(
                            "String `{}` is a valid Turbo Pascal var declaration",
                            self.source
                        );
                        self.error = String::new();

                        for (identifier, typ) in &identifiers {
                            self.identifiers.push_str(
                                format!("Identifier: {}, type: {}\n", identifier, typ).as_str(),
                            );
                        }
                    }
                    Err(e) => {
                        self.error = format!("{}", e);

                        self.before_error = self.source[0..e.pos()].to_string();
                        if e.pos() != self.source.len() {
                            self.error_token =
                                self.source[e.pos()..e.pos() + e.tok_length()].to_string();
                            self.after_error = self.source
                                [e.pos() + e.tok_length()..self.source.len()]
                                .to_string();
                        } else {
                            self.error_token = String::from(";");
                            self.after_error = String::new();
                        }
                    }
                },
                Err(e) => self.error = format!("{}", e),
            },
        }
    }

    pub fn clear(&mut self) {
        self.success = String::new();
        self.error = String::new();

        self.before_error = String::new();
        self.error_token = String::new();
        self.after_error = String::new();

        self.identifiers = String::new();
    }
}

fn main() -> iced::Result {
    iced::run("Analyzer | Turbo Pascal VAR", App::update, App::view)
}

// use analyzer::{analyze, tokenize};

// fn main() {
//     let content: String = String::from("VAR A,K:ARRAY[2:10,10:40] OF BYTE, D17,E7 : WORD;");

//     match tokenize(content.clone()) {
//         Ok(tokens) => match analyze(tokens) {
//             Ok(()) => {
//                 println!(
//                     "String `{}` is a valid Turbo Pascal var declaration",
//                     content
//                 );
//             }
//             Err(e) => {
//                 println!("{}", e)
//             }
//         },
//         Err(e) => println!("{}", e),
//     }
// }
