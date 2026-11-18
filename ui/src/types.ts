export type HttpMethod = "get" | "post" | "put" | "patch" | "delete";

export type RequestDraft = {
  method: HttpMethod;
  url: string;
  headers: [string, string][];
  body: string;
};

export type ResponseSnapshot = {
  status: number;
  headers: [string, string][];
  body: string;
  elapsed_ms: number;
};

export type Room = {
  id: string;
  name: string;
  base_url: string;
  ms: number;
};

export type SavedRequest = {
  id: string;
  room_id: string;
  name: string;
  draft: RequestDraft;
  response: ResponseSnapshot;
  ms: number;
};

export type Bootstrap = {
  rooms: Room[];
  requests: SavedRequest[];
  current_room_id: string;
  error: string | null;
};