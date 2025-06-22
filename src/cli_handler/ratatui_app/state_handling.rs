use anyhow::Context;
use crossterm::event::{KeyCode, KeyModifiers};
use futures_util::StreamExt;
use tui_textarea::TextArea;

use crate::provider::{ChatData, ChatHistoryItem, Provider};

use super::{
    App, Popover, SelectedZone,
    event_handler::{Event, LlmResponse},
    input::Input,
};

enum WindowDirection {
    Up,
    Down,
}

impl<'a, 't> App<'a, 't> {
    pub(super) fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    self.handle_key_event(key_event)?;
                }
            }
            Event::LlmResponse(LlmResponse::Chunk(chunk)) => {
                if let Some(index) = self.last_added_index {
                    self.provider.append_chat_in_context(index, &chunk)?;
                } else {
                    self.last_added_index = self
                        .provider
                        .add_chat_to_context(ChatHistoryItem::Chat(ChatData::model(chunk)))?;
                }
            }
            Event::LlmResponse(LlmResponse::Finished) => {
                self.generating = false;
                self.last_added_index = None;
            }
            Event::Error(err) => {
                if err.root_cause().is::<std::io::Error>() {
                    return Err(err).context("Critical Event Handling Error. Exiting as keyboard inputs could fail to exit program.");
                }
                self.generating = false;
                self.last_added_index = None;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> anyhow::Result<()> {
        // Global
        let input = Input::from((key_event, self.selected_zone));
        match input {
            Input::Quit => self.exit(),
            Input::ChangeWindowUp => self.change_window(WindowDirection::Up),
            Input::ChangeWindowDown => self.change_window(WindowDirection::Down),
            Input::ScrollUp => {
                self.scroll_chat_history(WindowDirection::Up);
            }
            Input::ScrollDown => {
                self.scroll_chat_history(WindowDirection::Down);
            }
            Input::Submit => {
                self.submit_prompt()?;
            }

            Input::TextAreaInput(key_event) => {
                if !self.generating {
                    self.textarea.input(key_event);
                }
            }
            Input::Back => {
                self.popover = None;
            }
            Input::OpenLlmOptions => {
                self.popover = Some(Popover::LlmToolList);
            }
            Input::None => {}
        };
        Ok(())
    }

    fn submit_prompt(&mut self) -> anyhow::Result<()> {
        let prompt = self.textarea.lines().join("\n");
        self.provider
            .add_chat_to_context(ChatHistoryItem::Chat(ChatData::user(prompt.clone())))?;
        self.textarea = TextArea::default();
        self.generating = true;
        tokio::spawn(handle_llm_stream(
            self.event_handler.get_sender(),
            self.provider.clone(),
            prompt,
        ));
        Ok(())
    }

    fn scroll_chat_history(&mut self, directon: WindowDirection) {
        match directon {
            WindowDirection::Up => {
                self.scrollview_state.scroll_up();
            }
            WindowDirection::Down => {
                self.scrollview_state.scroll_down();
            }
        };
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn change_window(&mut self, direction: WindowDirection) {
        match direction {
            WindowDirection::Up => match self.selected_zone {
                SelectedZone::ChatHistory => {
                    self.selected_zone = SelectedZone::TextInput;
                }
                SelectedZone::TextInput => {
                    self.selected_zone = SelectedZone::ChatHistory;
                }
            },
            WindowDirection::Down => match self.selected_zone {
                SelectedZone::ChatHistory => {
                    self.selected_zone = SelectedZone::TextInput;
                }
                SelectedZone::TextInput => {
                    self.selected_zone = SelectedZone::ChatHistory;
                }
            },
        }
    }
}

async fn handle_llm_stream(
    tx: tokio::sync::mpsc::UnboundedSender<Event>,
    mut provider: Provider,
    prompt: String,
) {
    let stream = provider.complete_chat_stream(prompt).await;
    let mut stream = match stream {
        Ok(stream) => stream,
        Err(err) => {
            let _ = tx.send(Event::Error(err));
            return;
        }
    };

    while let Some(response) = stream.next().await {
        match response {
            Ok(chunk) => {
                if tx
                    .send(Event::LlmResponse(LlmResponse::Chunk(chunk)))
                    .is_err()
                {
                    break; // Exit if the receiver is closed
                }
            }
            Err(e) => {
                if tx
                    .send(Event::Error(anyhow::anyhow!("LLM Stream error. {}", e)))
                    .is_err()
                {
                    break; // Exit if the receiver is closed
                }
            }
        }
    }
    let _ = tx.send(Event::LlmResponse(LlmResponse::Finished));
}
