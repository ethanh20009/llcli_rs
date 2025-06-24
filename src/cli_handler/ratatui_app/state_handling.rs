use anyhow::Context;
use crossterm::event::{KeyCode, KeyModifiers};
use futures_util::StreamExt;
use tracing::trace;
use tui_textarea::TextArea;

use crate::{
    cli_handler::ratatui_app::tool_list_popover::LlmToolEnum,
    provider::{ChatData, ChatHistoryItem, Provider},
};

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
                tracing::trace!("Handling Key Event: {:?}", key_event);
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    self.handle_key_event(key_event)?;
                }
            }
            Event::LlmResponse(LlmResponse::Chunk(chunk)) => {
                tracing::trace!("Handling LLM Response Chunk: {:?}", chunk);
                if let Some(index) = self.last_added_index {
                    self.provider.append_chat_in_context(index, &chunk)?;
                } else {
                    self.last_added_index = self
                        .provider
                        .add_chat_to_context(ChatHistoryItem::Chat(ChatData::model(chunk)))?;
                }
            }
            Event::LlmResponse(LlmResponse::Finished) => {
                tracing::trace!("Handling LLM Response Finished");
                self.generating = false;
                self.last_added_index = None;
            }
            Event::Error(err) => {
                tracing::error!("Error occurred: {:?}", err);
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
        let input = Input::from(key_event);
        trace!("Decoded input: {:?}", input);

        // Popover first.
        if let Some(popover) = self.popover {
            let handled = match input {
                Input::Back => {
                    self.popover = None;
                    true
                }
                Input::Quit => {
                    self.exit();
                    true
                }
                _ => false,
            };
            if handled {
                return Ok(());
            }
            let handled = match popover {
                Popover::LlmToolList => match input {
                    Input::ScrollUp => {
                        self.llm_tool_options_state.select_previous();
                        true
                    }
                    Input::ScrollDown => {
                        self.llm_tool_options_state.select_next();
                        true
                    }
                    Input::Toggle => {
                        if let Some(selected) = self.llm_tool_options_state.selected() {
                            self.provider.flags_mut().toggle(
                                LlmToolEnum::from_repr(selected)
                                    .context("Failed to get LLM Tool Setting from index.")?,
                            );
                        }
                        true
                    }
                    _ => false,
                },
            };
            if handled {
                return Ok(());
            }
        }

        // Normal global keybinds
        let handled_global = match input {
            Input::Quit => {
                self.exit();
                true
            }
            Input::ChangeWindowUp | Input::ChangeWindowDown => {
                self.change_window(if input == Input::ChangeWindowUp {
                    WindowDirection::Up
                } else {
                    WindowDirection::Down
                });
                true
            }
            Input::ToggleLlmOptions => {
                if let Some(Popover::LlmToolList) = self.popover {
                    self.popover = None
                } else {
                    self.popover = Some(Popover::LlmToolList);
                }
                true
            }
            _ => false,
        };
        if handled_global {
            return Ok(());
        }

        match self.selected_zone {
            SelectedZone::TextInput => match input {
                Input::Submit => {
                    self.submit_prompt()?;
                }
                _ => {
                    self.textarea.input(key_event);
                }
            },
            SelectedZone::ChatHistory => match input {
                Input::ScrollUp => self.scroll_chat_history(WindowDirection::Up),
                Input::ScrollDown => self.scroll_chat_history(WindowDirection::Down),
                Input::Submit => {
                    if !self.generating {
                        self.submit_prompt()?;
                    }
                }
                _ => {}
            },
        }

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
