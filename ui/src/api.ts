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

export type LoadProfile =
  | { kind: "fixed"; concurrency: number }
  | { kind: "ramp_up"; start: number; max: number; ramp_ms: number };

export type LoadReport = {
  profile: LoadProfile;
  total_vu: number;
  counted_ok: number;
  counted_failed: number;
  elapsed_ms: number;
  rps: number;
  min_ms: number;
  avg_ms: number;
  p95_ms: number;
  p99_ms: number;
  statuses: [number, number][];
  errors: string[];
};

export async function runLoad(payload: {
  baseUrl: string;
  total: number;
  profile: LoadProfile;
  draft: RequestDraft;
}): Promise<LoadReport> {
  return invoke("run_load", {
    args: {
      base_url: payload.baseUrl,
      total_vu: payload.total,
      profile: payload.profile,
      draft: payload.draft,
    },
  });
}

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