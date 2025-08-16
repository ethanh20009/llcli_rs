use anyhow::{Context, anyhow};
use futures_util::FutureExt;
use futures_util::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::{sync::mpsc, task::JoinHandle};

#[derive(Debug)]
pub struct EventHandler {
    tx: tokio::sync::mpsc::UnboundedSender<Event>,
    rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
    task: Option<JoinHandle<()>>,
}

#[derive(Debug)]
pub enum Event {
    Tick,
    Key(crossterm::event::KeyEvent),
    LlmResponse(LlmResponse),
    Error(anyhow::Error),
}

#[derive(Debug)]
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
                      Some(Err(err)) => {
                        _tx.send(Event::Error(anyhow!("Error reading terminal event. {}", err))).unwrap();
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

    pub fn get_sender(&self) -> UnboundedSender<Event> {
        self.tx.clone()
    }
}
