use std::sync::Mutex;

use orphey::backend::Backend;
use orphey::error::Error;
use orphey::models::load::LoadReport;
use orphey::models::request::{RequestDraft, ResponseSnapshot, SavedRequest};
use orphey::models::room::Room;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

pub struct AppState(pub Mutex<Backend>);

fn parse_uuid(raw: &str, label: &str) -> Result<Uuid, String> {
    Uuid::parse_str(raw).map_err(|err| format!("{label}: {err}"))
}

fn err_str(err: Error) -> String {
    err.to_string()
}

#[derive(Serialize)]
pub struct Bootstrap {
    pub rooms: Vec<Room>,
    pub requests: Vec<SavedRequest>,
    pub current_room_id: String,
    pub error: Option<String>,
}

#[tauri::command(rename_all = "snake_case")]
pub fn bootstrap(state: State<AppState>) -> Bootstrap {
    let backend = state.0.lock().expect("backend lock");
    let mut error = None;

    let rooms = match backend.list_rooms() {
        Ok(rooms) => rooms,
        Err(err) => {
            error = Some(err_str(err));
            Vec::new()
        }
    };

    let current_room_id = rooms
        .first()
        .map(|r| r.id.to_string())
        .unwrap_or_else(|| Uuid::nil().to_string());

    let room_uuid = rooms.first().map(|r| r.id).unwrap_or_else(Uuid::nil);
    let requests = match backend.list_requests(room_uuid) {
        Ok(requests) => requests,
        Err(err) => {
            if error.is_none() {
                error = Some(err_str(err));
            }
            Vec::new()
        }
    };

    Bootstrap {
        rooms,
        requests,
        current_room_id,
        error,
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn list_requests(state: State<AppState>, room_id: String) -> Result<Vec<SavedRequest>, String> {
    let backend = state.0.lock().expect("backend lock");
    let id = parse_uuid(&room_id, "room_id")?;
    backend.list_requests(id).map_err(err_str)
}

#[derive(Deserialize)]
pub struct CreateRoomArgs {
    pub name: String,
    pub base_url: String,
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_room(state: State<AppState>, args: CreateRoomArgs) -> Result<Room, String> {
    let backend = state.0.lock().expect("backend lock");
    backend
        .create_room(args.name, args.base_url)
        .map_err(err_str)
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_room(state: State<AppState>, room_id: String) -> Result<(), String> {
    let backend = state.0.lock().expect("backend lock");
    let id = parse_uuid(&room_id, "room_id")?;
    backend.delete_room(id).map_err(err_str)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_room_base_url(
    state: State<AppState>,
    room_id: String,
    base_url: String,
) -> Result<(), String> {
    let backend = state.0.lock().expect("backend lock");
    let id = parse_uuid(&room_id, "room_id")?;
    backend.set_room_base_url(id, base_url).map_err(err_str)
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_request(state: State<AppState>, request_id: String) -> Result<(), String> {
    let backend = state.0.lock().expect("backend lock");
    let id = parse_uuid(&request_id, "request_id")?;
    backend.delete_request(id).map_err(err_str)
}

#[derive(Deserialize)]
pub struct SendRequestArgs {
    pub room_id: String,
    pub selected_id: Option<String>,
    pub name: String,
    pub base_url: String,
    pub draft: RequestDraft,
}

#[derive(Serialize)]
pub struct SendRequestResult {
    pub saved: SavedRequest,
    pub snapshot: ResponseSnapshot,
}

#[tauri::command(rename_all = "snake_case")]
pub fn send_request(
    state: State<AppState>,
    args: SendRequestArgs,
) -> Result<SendRequestResult, String> {
    let backend = state.0.lock().expect("backend lock");
    let room_id = parse_uuid(&args.room_id, "room_id")?;
    let selected_id = match &args.selected_id {
        Some(raw) => Some(parse_uuid(raw, "selected_id")?),
        None => None,
    };

    let snapshot = backend
        .send_http(args.draft.clone(), &args.base_url)
        .map_err(err_str)?;

    let saved = backend
        .save_exchange(
            room_id,
            selected_id,
            &args.name,
            args.draft,
            snapshot.clone(),
        )
        .map_err(err_str)?;

    Ok(SendRequestResult { saved, snapshot })
}

#[derive(Deserialize)]
pub struct LoadArgs {
    pub base_url: String,
    pub concurrency: usize,
    pub total_vu: usize,
    pub draft: RequestDraft,
}

#[tauri::command(rename_all = "snake_case")]
pub fn run_load(state: State<AppState>, args: LoadArgs) -> Result<LoadReport, String> {
    let backend = state.0.lock().expect("backend lock");
    backend
        .run_load(args.draft, &args.base_url, args.concurrency, args.total_vu)
        .map_err(|err| err.to_string())
}
