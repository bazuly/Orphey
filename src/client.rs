use std::time::Duration;

use crate::error::Error;
use reqwest::header::{HeaderName, HeaderValue};
use tokio::time::Instant;

use crate::models::rest::{Method, RequestDraft, ResponseSnapshot};

pub async fn send(draft: RequestDraft) -> Result<ResponseSnapshot, Error> {
    let client = create_client()?;
    let request = build_request(draft, &client)?;

    execute(&client, request).await
}

pub fn build_request(
    draft: RequestDraft,
    client: &reqwest::Client,
) -> Result<reqwest::Request, Error> {
    // проверяем url из draft на валидность
    let url = reqwest::Url::parse(&draft.url).map_err(|err| Error::Url(err.to_string()))?;

    let mut builder = client.request(http_method(&draft.method), url);

    for (name, value) in &draft.headers {
        if name.trim().is_empty() {
            continue;
        }

        let name =
            HeaderName::from_bytes(name.as_bytes()).map_err(|err| Error::Url(err.to_string()))?;
        let value = HeaderValue::from_str(value).map_err(|err| Error::Url(err.to_string()))?;

        builder = builder.header(name, value);
    }

    if sends_body(&draft.method) {
        builder = builder.body(draft.body);
    }

    let builder = builder.build().map_err(|err| Error::Url(err.to_string()))?;

    Ok(builder)
}

pub async fn execute(
    client: &reqwest::Client,
    request: reqwest::Request,
) -> Result<ResponseSnapshot, Error> {
    let started = Instant::now();

    let response = client
        .execute(request)
        .await
        .map_err(|err| Error::Network(err.to_string()))?;

    let status = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_string(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect();

    let body = response
        .text()
        .await
        .map_err(|err| Error::Network(err.to_string()))?;

    Ok(ResponseSnapshot {
        status,
        headers,
        body,
        elapsed_ms: started.elapsed().as_millis() as u64,
    })
}

fn create_client() -> Result<reqwest::Client, Error> {
    // создаем клиента для http запроса
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|err| Error::Network(err.to_string()))?;

    Ok(client)
}

// по навзания проверяем и возвращаем нужный рест метод из reqwest
fn http_method(method: &Method) -> reqwest::Method {
    match method {
        Method::GET => reqwest::Method::GET,
        Method::POST => reqwest::Method::POST,
        Method::PATCH => reqwest::Method::PATCH,
        Method::PUT => reqwest::Method::PUT,
        Method::DELETE => reqwest::Method::DELETE,
    }
}
// если пост, пут или патч
// то прикрепляем тело к ответу
fn sends_body(method: &Method) -> bool {
    matches!(method, Method::POST | Method::PUT | Method::PATCH)
}
