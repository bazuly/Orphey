use crate::error::Error;
use crate::models::rest::{RequestDraft, ResponseSnapshot};
use std::sync::mpsc::{Receiver, TryRecvError};

pub struct AppService {
    rt: tokio::runtime::Runtime,
    inflight: Option<(RequestDraft, Receiver<Result<ResponseSnapshot, Error>>)>,
}

impl AppService {
    pub fn new() -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
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

        let sent = draft.clone();
        self.rt.spawn(async move {
            let result = crate::client::send(draft).await;
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        self.inflight = Some((sent, rx));
        true
    }

    pub fn poll(
        &mut self,
        ctx: &egui::Context,
    ) -> Option<Result<(RequestDraft, ResponseSnapshot), Error>> {
        // берем inflight as_ref с помощью try_recv и проверяем его
        // пришло ли что-то или нет
        let message = self.inflight.as_ref().map(|rx| rx.1.try_recv());

        match message {
            Some(Ok(Ok(snapshot))) => {
                let (draft, _) = self.inflight.take().unwrap();
                Some(Ok((draft, snapshot)))
            }
            Some(Ok(Err(err))) => {
                self.inflight = None;
                Some(Err(err))
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
