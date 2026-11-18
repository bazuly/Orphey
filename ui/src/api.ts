import { invoke } from "@tauri-apps/api/core";
import type { Bootstrap, RequestDraft, Room, SavedRequest, ResponseSnapshot } from "./types";

export async function bootstrap(): Promise<Bootstrap> {
  return invoke("bootstrap");
}

export async function listRequests(roomId: string): Promise<SavedRequest[]> {
  return invoke("list_requests", { room_id: roomId });
}

export async function createRoom(name: string, baseUrl: string): Promise<Room> {
  return invoke("create_room", { args: { name, base_url: baseUrl } });
}

export async function deleteRoom(roomId: string): Promise<void> {
  return invoke("delete_room", { room_id: roomId });
}

export async function setRoomBaseUrl(roomId: string, baseUrl: string): Promise<void> {
  return invoke("set_room_base_url", { room_id: roomId, base_url: baseUrl });
}

export async function deleteRequest(requestId: string): Promise<void> {
  return invoke("delete_request", { request_id: requestId });
}

export type SendResult = { saved: SavedRequest; snapshot: ResponseSnapshot };

export async function sendRequest(payload: {
  roomId: string;
  selectedId: string | null;
  name: string;
  baseUrl: string;
  draft: RequestDraft;
}): Promise<SendResult> {
  return invoke("send_request", {
    args: {
      room_id: payload.roomId,
      selected_id: payload.selectedId,
      name: payload.name,
      base_url: payload.baseUrl,
      draft: payload.draft,
    },
  });
}