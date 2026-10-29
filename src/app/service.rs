use crate::error::Error;
use crate::models::rest::{RequestDraft, ResponseSnapshot};
use std::sync::mpsc::{Receiver, TryRecvError};

pub struct AppService {
    rt: tokio::runtime::Runtime,
    inflight: Option<Receiver<Result<ResponseSnapshot, Error>>>,
}

impl AppService {
    pub fn new() -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .build()
            .expect("tokio rt");

        Self { rt, inflight: None }
    }

    // begin возвращает false, если запрос уже летит.
    // Статус сервис не пишет: он не знает про окно. poll отдаёт None,
    // пока ответа нет, и сам просит новый кадр.

    pub fn begin(&mut self, draft: RequestDraft, ctx: egui::Context) -> bool {
        // если уже что-то отправили
        // то возвращаем false
        if self.inflight.is_some() {
            return false;
        }

        // создаем поток и отправляем в него draft
        let (tx, rx) = std::sync::mpsc::channel();

        self.rt.spawn(async move {
            let result = crate::client::send(draft).await;
            let _ = tx.send(result);
            // анимация ui, что что-то произошло
            ctx.request_repaint();
        });
        self.inflight = Some(rx);
        true
    }

    pub fn poll(&mut self, ctx: &egui::Context) -> Option<Result<ResponseSnapshot, Error>> {
        // берем inflight as_ref с помощью try_recv и проверяем его
        // пришло ли что-то или нет
        let message = self.inflight.as_ref().map(|rx| rx.try_recv());

        match message {
            Some(Ok(result)) => {
                self.inflight = None;
                Some(result)
            }
            Some(Err(TryRecvError::Empty)) => {
                ctx.request_repaint();
                None
            }
            Some(Err(TryRecvError::Disconnected)) => {
                self.inflight = None;
                Some(Err(Error::Network("Request disconnected..".to_string())))
            }
            None => None,
        }
    }
}
