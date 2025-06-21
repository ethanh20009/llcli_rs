use anyhow::Context;
use futures_util::FutureExt;
use futures_util::StreamExt;
use tokio::{sync::mpsc, task::JoinHandle};

#[derive(Debug)]
pub struct EventHandler {
    pub tx: tokio::sync::mpsc::UnboundedSender<Event>,
    pub rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
    pub task: Option<JoinHandle<()>>,
}

pub enum Event {
    Tick,
    Key(crossterm::event::KeyEvent),
    LlmResponse(LlmResponse),
    Error,
}

pub enum LlmResponse {
    Finished,
    Chunk(String),
}

impl EventHandler {
    pub fn new() -> Self {
        let tick_rate = std::time::Duration::from_millis(250);

        let (_tx, rx) = mpsc::unbounded_channel();
        let tx = _tx.clone();

        let task = tokio::spawn(async move {
            let mut reader = ratatui::crossterm::event::EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                let delay = interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  maybe_event = crossterm_event => {
                    match maybe_event {
                      Some(Ok(evt)) => {
                        match evt {
                          crossterm::event::Event::Key(key) => {
                            if key.kind == crossterm::event::KeyEventKind::Press {
                              _tx.send(Event::Key(key)).unwrap();
                            }
                          },
                          _ => {},
                        }
                      }
                      Some(Err(_)) => {
                        _tx.send(Event::Error).unwrap();
                      }
                      None => {},
                    }
                  },
                  _ = delay => {
                      _tx.send(Event::Tick).unwrap();
                  },
                }
            }
        });

        Self {
            tx,
            rx,
            task: Some(task),
        }
    }

    pub async fn next(&mut self) -> anyhow::Result<Event> {
        self.rx.recv().await.context("Unable to get event.")
    }

    pub fn send_llm_response(&self, response: LlmResponse) -> anyhow::Result<()> {
        self.tx
            .send(Event::LlmResponse(response))
            .context("Failed to send LLM response")?;
        Ok(())
    }
}
