use std::{
    collections::HashMap,
    io::{self, Write},
};

use anyhow::Result;
use bat::WrappingMode;
use cliclack::spinner;
use goose::models::message::{Message, MessageContent, ToolRequest, ToolResponse};

use super::prompt::{Input, InputType, Prompt, Theme};

pub struct RustylinePrompt {
    spinner: cliclack::ProgressBar,
    theme: Theme,
    renderers: HashMap<String, Box<dyn ToolRenderer>>,
}

impl RustylinePrompt {
    pub fn new() -> Self {
        // // Load highlighting assets
        // let assets = HighlightingAssets::from_binary();

        // // Fetch and list all available themes
        // let themes = assets.themes();
        // for theme_name in themes {
        //     println!("{}", theme_name);
        // }

        // // List all available syntaxes (languages)
        // let syntax_set = assets.get_syntaxes().unwrap();
        // for syntax in syntax_set {
        //     println!("{}", syntax.name);
        // }

        let mut renderers: HashMap<String, Box<dyn ToolRenderer>> = HashMap::new();
        let default_renderer = DefaultRenderer;
        renderers.insert(default_renderer.tool_name(), Box::new(default_renderer));

        RustylinePrompt {
            spinner: spinner(),
            theme: Theme::Dark,
            renderers,
        }
    }
}

/// Implement the ToolRenderer trait for each tool that you want to render in the prompt.
trait ToolRenderer {
    fn tool_name(&self) -> String;
    fn request(&self, tool_request: &ToolRequest, theme: &str);
    fn response(&self, tool_response: &ToolResponse, theme: &str);
}

struct DefaultRenderer;

impl ToolRenderer for DefaultRenderer {
    fn tool_name(&self) -> String {
        "default".to_string()
    }

    fn request(&self, tool_request: &ToolRequest, theme: &str) {
        match &tool_request.tool_call {
            Ok(call) => {
                print_tool_request(
                    &serde_json::to_string_pretty(&call.arguments).unwrap(),
                    theme,
                    &call.name,
                );
            }
            Err(e) => print(&e.to_string(), theme),
        }
    }

    fn response(&self, tool_response: &ToolResponse, theme: &str) {
        match &tool_response.tool_result {
            Ok(output) => {
                let output_value = serde_json::to_string_pretty(output).unwrap();

                // For pure text responses, strip the quotes and replace escaped newlines. Eg. bash responses
                let unquoted = output_value.trim_matches('"');
                let formatted = unquoted.replace("\\n", "\n");

                let language = if formatted.starts_with("{") {
                    "JSON"
                } else {
                    "Markdown"
                };
                print_tool_response(&formatted, theme, language);
            }
            Err(e) => print(&e.to_string(), theme),
        }
    }
}

fn print_tool_request(content: &str, theme: &str, tool_name: &str) {
    bat::PrettyPrinter::new()
        .input(
            bat::Input::from_bytes(content.as_bytes()).name(format!("Tool Request: {}", tool_name)),
        )
        .theme(theme)
        .language("JSON")
        .grid(true)
        .header(true)
        .wrapping_mode(WrappingMode::Character)
        .print()
        .unwrap();
}

fn print_tool_response(content: &str, theme: &str, language: &str) {
    bat::PrettyPrinter::new()
        .input(bat::Input::from_bytes(content.as_bytes()).name("Tool Response:"))
        .theme(theme)
        .language(language)
        .grid(true)
        .header(true)
        .wrapping_mode(WrappingMode::Character)
        .print()
        .unwrap();
}

fn print(content: &str, theme: &str) {
    bat::PrettyPrinter::new()
        .input(bat::Input::from_bytes(content.as_bytes()))
        .theme(theme)
        .language("Markdown")
        .wrapping_mode(WrappingMode::Character)
        .print()
        .unwrap();
}

fn print_newline() {
    println!();
}

impl Prompt for RustylinePrompt {
    fn render(&mut self, message: Box<Message>) {
        let theme = match self.theme {
            Theme::Light => "GitHub",
            Theme::Dark => "zenburn",
        };

        let mut last_tool_name: &str = "default";
        for message_content in &message.content {
            match message_content {
                MessageContent::Text(text) => print(&text.text, theme),
                MessageContent::ToolRequest(tool_request) => match &tool_request.tool_call {
                    Ok(call) => {
                        last_tool_name = &call.name;
                        self.renderers
                            .get(&call.name)
                            .or_else(|| self.renderers.get("default"))
                            .unwrap()
                            .request(tool_request, theme);
                    }
                    Err(_) => self
                        .renderers
                        .get("default")
                        .unwrap()
                        .request(tool_request, theme),
                },
                MessageContent::ToolResponse(tool_response) => self
                    .renderers
                    .get(last_tool_name)
                    .or_else(|| self.renderers.get("default"))
                    .unwrap()
                    .response(tool_response, theme),
                MessageContent::Image(image) => {
                    println!("Image: [data: {}, type: {}]", image.data, image.mime_type);
                }
            }
        }

        print_newline();
        io::stdout().flush().expect("Failed to flush stdout");
    }

    fn show_busy(&mut self) {
        self.spinner = spinner();
        self.spinner.start("Thinking...");
    }

    fn hide_busy(&self) {
        self.spinner.stop("");
    }

    fn get_input(&mut self) -> Result<Input> {
        let mut editor = rustyline::DefaultEditor::new()?;
        let input = editor.readline("( O)> ");
        let mut message_text = match input {
            Ok(text) => text,
            Err(e) => {
                match e {
                    rustyline::error::ReadlineError::Interrupted => (),
                    _ => eprintln!("Input error: {}", e),
                }
                return Ok(Input {
                    input_type: InputType::Exit,
                    content: None,
                });
            }
        };
        message_text = message_text.trim().to_string();

        if message_text.eq_ignore_ascii_case("/exit") || message_text.eq_ignore_ascii_case("/quit")
        {
            Ok(Input {
                input_type: InputType::Exit,
                content: None,
            })
        } else if message_text.eq_ignore_ascii_case("/t") {
            self.theme = match self.theme {
                Theme::Light => {
                    println!("Switching to Dark theme");
                    Theme::Dark
                }
                Theme::Dark => {
                    println!("Switching to Light theme");
                    Theme::Light
                }
            };
            return Ok(Input {
                input_type: InputType::AskAgain,
                content: None,
            });
        } else if message_text.eq_ignore_ascii_case("/?")
            || message_text.eq_ignore_ascii_case("/help")
        {
            println!("Commands:");
            println!("/exit - Exit the session");
            println!("/t - Toggle Light/Dark theme");
            println!("/? | /help - Display this help message");
            println!("Ctrl+C - Interrupt goose (resets the interaction to before the interrupted user request)");
            return Ok(Input {
                input_type: InputType::AskAgain,
                content: None,
            });
        } else {
            return Ok(Input {
                input_type: InputType::Message,
                content: Some(message_text.to_string()),
            });
        }
    }

    fn close(&self) {
        // No cleanup required
    }
}