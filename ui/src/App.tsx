import { useCallback, useEffect, useMemo, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import * as api from "./api";
import type { HttpMethod, RequestDraft, ResponseSnapshot, Room, SavedRequest } from "./types";

const METHODS: HttpMethod[] = ["get", "post", "put", "patch", "delete"];

const COMMON_HEADERS = [
  "Accept",
  "Accept-Encoding",
  "Accept-Language",
  "Authorization",
  "Cache-Control",
  "Content-Type",
  "Cookie",
  "Origin",
  "User-Agent",
  "X-API-Key",
  "X-Request-ID",
];

const COMMON_VALUES = [
  "application/json",
  "application/x-www-form-urlencoded",
  "multipart/form-data",
  "text/plain",
  "Bearer ",
  "Basic ",
  "no-cache",
  "gzip, deflate, br",
];

type QueryParam = { name: string; value: string };

type Pair = [string, string];

type EditorTab = {
  key: string;
  savedId: string | null;
  roomId: string;
  name: string;
  draft: RequestDraft;
  response: ResponseSnapshot | null;
  loadReport: api.LoadReport | null;
};

const defaultDraft = (): RequestDraft => ({
  method: "get",
  url: "",
  headers: [],
  body: '{\n  "id": 1\n}',
});

function methodBodyTemplate(method: HttpMethod): string {
  switch (method) {
    case "get":
      return '{\n  "id": 1\n}';
    case "post":
      return '{\n  "name": "Ada",\n  "email": "ada@example.com"\n}';
    case "put":
      return '{\n  "id": 1,\n  "name": "Ada",\n  "email": "ada@example.com"\n}';
    case "patch":
      return '{\n  "name": "Ada Lovelace"\n}';
    case "delete":
      return '{\n  "id": 1\n}';
  }
}

function displayName(name: string, draft: RequestDraft): string {
  const custom = name.trim();
  const prefix = `${draft.method.toUpperCase()} `;
  if (custom.toUpperCase().startsWith(prefix)) {
    const rest = custom.slice(prefix.length).trim();
    if (rest === draft.url.trim()) return rest;
  }
  return custom;
}

function requestLabel(saved: SavedRequest): string {
  return displayName(saved.name, saved.draft) || saved.draft.url.trim() || "Untitled";
}

function tabLabel(tab: EditorTab): string {
  return tab.name.trim() || tab.draft.url.trim() || "Untitled";
}

function splitUrl(url: string): { path: string; params: QueryParam[] } {
  const q = url.indexOf("?");
  if (q < 0) return { path: url, params: [] };
  const path = url.slice(0, q);
  const raw = url.slice(q + 1);
  if (!raw) return { path, params: [] };
  const params = raw.split("&").map((part) => {
    const eq = part.indexOf("=");
    if (eq < 0) return { name: decodeURIComponent(part), value: "" };
    return {
      name: decodeURIComponent(part.slice(0, eq)),
      value: decodeURIComponent(part.slice(eq + 1)),
    };
  });
  return { path, params };
}

function joinUrl(path: string, params: QueryParam[]): string {
  const query = params
    .filter((param) => param.name.trim() || param.value.trim())
    .map((param) => `${encodeURIComponent(param.name)}=${encodeURIComponent(param.value)}`)
    .join("&");
  return query ? `${path}?${query}` : path;
}

function statusText(code: number): string {
  const names: Record<number, string> = {
    200: "OK",
    201: "Created",
    204: "No Content",
    301: "Moved",
    302: "Found",
    400: "Bad Request",
    401: "Unauthorized",
    403: "Forbidden",
    404: "Not Found",
    409: "Conflict",
    422: "Unprocessable",
    500: "Internal Server Error",
    502: "Bad Gateway",
    503: "Unavailable",
  };
  return names[code] ? `${code} ${names[code]}` : String(code);
}

function formatElapsed(ms: number): string {
  if (ms >= 1000) return `${(ms / 1000).toFixed(1)}s`;
  return `${ms} ms`;
}

function reportJson(report: api.LoadReport): string {
  return JSON.stringify(report, null, 2);
}

function releaseFocus() {
  const focused = document.activeElement;
  if (focused instanceof HTMLElement) focused.blur();
}

function keepWindowInFront() {
  setTimeout(() => {
    void getCurrentWindow().setFocus().catch(() => undefined);
  }, 0);
}

function formatSize(text: string): string {
  const bytes = new TextEncoder().encode(text).length;
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default function App() {
  const [rooms, setRooms] = useState<Room[]>([]);
  const [requests, setRequests] = useState<SavedRequest[]>([]);
  const [currentRoomId, setCurrentRoomId] = useState("");
  const [openRooms, setOpenRooms] = useState<string[]>([]);
  const [tabs, setTabs] = useState<EditorTab[]>([]);
  const [activeKey, setActiveKey] = useState<string | null>(null);

  const [roomNameDraft, setRoomNameDraft] = useState("");
  const [creatingRoom, setCreatingRoom] = useState(false);
  const [query, setQuery] = useState("");
  const [roomBaseUrl, setRoomBaseUrl] = useState("");

  const [status, setStatus] = useState("");
  const [sending, setSending] = useState(false);
  const [loading, setLoading] = useState(false);
  const [profileKind, setProfileKind] = useState<"fixed" | "ramp_up" | "spike">("fixed");
  const [concurrency, setConcurrency] = useState(10);
  const [rampStart, setRampStart] = useState(1);
  const [rampSeconds, setRampSeconds] = useState(10);
  const [spikeBase, setSpikeBase] = useState(2);
  const [spikeAt, setSpikeAt] = useState(5);
  const [spikeFor, setSpikeFor] = useState(2);
  const [loadTotal, setLoadTotal] = useState(100);
  const [booting, setBooting] = useState(true);
  const [bootError, setBootError] = useState<string | null>(null);
  const [editorTab, setEditorTab] = useState<"params" | "body" | "headers">("params");
  const [responseTab, setResponseTab] = useState<"body" | "headers" | "load">("body");

  const currentRoom = useMemo(
    () => rooms.find((room) => room.id === currentRoomId),
    [rooms, currentRoomId],
  );
  const roomTabs = tabs.filter((tab) => tab.roomId === currentRoomId);
  const active = roomTabs.find((tab) => tab.key === activeKey) ?? null;

  const needle = query.trim().toLowerCase();
  const visibleRooms = rooms.filter((room) => {
    if (!needle) return true;
    if (room.name.toLowerCase().includes(needle)) return true;
    if (room.id !== currentRoomId) return false;
    return requests.some(
      (saved) =>
        requestLabel(saved).toLowerCase().includes(needle) ||
        saved.draft.method.includes(needle),
    );
  });
  const visibleRequests = requests.filter((saved) => {
    if (!needle) return true;
    return (
      requestLabel(saved).toLowerCase().includes(needle) ||
      saved.draft.method.includes(needle) ||
      (currentRoom?.name.toLowerCase().includes(needle) ?? false)
    );
  });

  const updateActive = (patch: (tab: EditorTab) => EditorTab) => {
    if (!active) return;
    const key = active.key;
    setTabs((prev) => prev.map((tab) => (tab.key === key ? patch(tab) : tab)));
  };

  const setDraft = (next: RequestDraft | ((prev: RequestDraft) => RequestDraft)) => {
    updateActive((tab) => ({
      ...tab,
      draft: typeof next === "function" ? next(tab.draft) : next,
    }));
  };

  const loadRequests = useCallback(async (roomId: string) => {
    if (!roomId) {
      setRequests([]);
      return;
    }
    try {
      const list = await api.listRequests(roomId);
      setRequests(list);
    } catch (err) {
      setStatus(String(err));
    }
  }, []);

  const refreshBootstrap = useCallback(async () => {
    const data = await api.bootstrap();
    setRooms(data.rooms);
    setCurrentRoomId(data.current_room_id);
    setRequests(data.requests);
    if (data.error) setStatus(data.error);
    const room = data.rooms.find((item) => item.id === data.current_room_id);
    if (room) {
      setRoomBaseUrl(room.base_url);
      setOpenRooms([room.id]);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    refreshBootstrap()
      .then(() => {
        if (!cancelled) {
          setBootError(null);
          setBooting(false);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          const message = String(err);
          setBootError(message);
          setStatus(message);
          setBooting(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [refreshBootstrap]);

  const focusRoomTab = (roomId: string) => {
    setActiveKey((key) => {
      const current = tabs.find((tab) => tab.key === key);
      if (current?.roomId === roomId) return key;
      const last = [...tabs].reverse().find((tab) => tab.roomId === roomId);
      return last?.key ?? null;
    });
  };

  const selectRoom = async (room: Room) => {
    if (room.id === currentRoomId) {
      setOpenRooms((prev) =>
        prev.includes(room.id) ? prev.filter((id) => id !== room.id) : [...prev, room.id],
      );
      return;
    }
    setCurrentRoomId(room.id);
    setRoomBaseUrl(room.base_url);
    setOpenRooms((prev) => (prev.includes(room.id) ? prev : [...prev, room.id]));
    focusRoomTab(room.id);
    await loadRequests(room.id);
  };

  const addRoom = async () => {
    const name = roomNameDraft.trim();
    if (!name) return;
    try {
      const room = await api.createRoom(name, "");
      setRooms((prev) => [...prev, room]);
      setRoomNameDraft("");
      setCreatingRoom(false);
      setCurrentRoomId(room.id);
      setRoomBaseUrl("");
      setOpenRooms((prev) => [...prev, room.id]);
      setRequests([]);
      setActiveKey(null);
    } catch (err) {
      setStatus(String(err));
    }
  };

  const removeRoom = async (roomId: string) => {
    try {
      await api.deleteRoom(roomId);
      const next = rooms.filter((room) => room.id !== roomId);
      setRooms(next);
      setTabs((prev) => prev.filter((tab) => tab.roomId !== roomId));
      setOpenRooms((prev) => prev.filter((id) => id !== roomId));
      if (currentRoomId === roomId) {
        const first = next[0];
        if (first) await selectRoom(first);
        else {
          setCurrentRoomId("");
          setRoomBaseUrl("");
          setRequests([]);
          setActiveKey(null);
        }
      }
    } catch (err) {
      setStatus(String(err));
    }
  };

  const saveBaseUrl = async () => {
    if (!currentRoomId) return;
    const base = roomBaseUrl.trim();
    if (currentRoom && currentRoom.base_url === base) return;
    try {
      await api.setRoomBaseUrl(currentRoomId, base);
      setRooms((prev) =>
        prev.map((room) => (room.id === currentRoomId ? { ...room, base_url: base } : room)),
      );
      setRoomBaseUrl(base);
    } catch (err) {
      setStatus(String(err));
    }
  };

  const openRequest = (saved: SavedRequest) => {
    const found = tabs.find((tab) => tab.savedId === saved.id);
    if (found) {
      setActiveKey(found.key);
      return;
    }
    const key = crypto.randomUUID();
    setTabs((prev) => [
      ...prev,
      {
        key,
        savedId: saved.id,
        roomId: saved.room_id,
        name: displayName(saved.name, saved.draft),
        draft: saved.draft,
        response: saved.response,
        loadReport: null,
      },
    ]);
    setActiveKey(key);
  };

  const newRequest = () => {
    if (!currentRoomId) return;
    const key = crypto.randomUUID();
    setTabs((prev) => [
      ...prev,
      {
        key,
        savedId: null,
        roomId: currentRoomId,
        name: "",
        draft: defaultDraft(),
        response: null,
        loadReport: null,
      },
    ]);
    setActiveKey(key);
    setEditorTab("params");
  };

  const closeTab = (key: string) => {
    setTabs((prev) => {
      const roomOnes = prev.filter((tab) => tab.roomId === currentRoomId);
      const idx = roomOnes.findIndex((tab) => tab.key === key);
      const next = prev.filter((tab) => tab.key !== key);
      if (key === activeKey) {
        const remaining = next.filter((tab) => tab.roomId === currentRoomId);
        const neighbor = remaining[idx - 1] ?? remaining[idx] ?? null;
        setActiveKey(neighbor?.key ?? null);
      }
      return next;
    });
  };

  const removeRequest = async (id: string) => {
    try {
      await api.deleteRequest(id);
      setRequests((prev) => prev.filter((saved) => saved.id !== id));
      setTabs((prev) => {
        const next = prev.filter((tab) => tab.savedId !== id);
        if (active?.savedId === id) {
          const remaining = next.filter((tab) => tab.roomId === currentRoomId);
          setActiveKey(remaining[remaining.length - 1]?.key ?? null);
        }
        return next;
      });
    } catch (err) {
      setStatus(String(err));
    }
  };

  const onMethodChange = (method: HttpMethod) => {
    setDraft((prev) => {
      const prevTemplate = methodBodyTemplate(prev.method);
      const next = { ...prev, method };
      if (!prev.body.trim() || prev.body === prevTemplate) {
        next.body = methodBodyTemplate(method);
      }
      return next;
    });
  };

  const runLoad = async () => {
    if (!currentRoomId || !active || sending || loading) return;
    const max = Math.min(50, Math.max(1, concurrency));
    const start = Math.min(max, Math.max(1, rampStart));
    const seconds = Math.min(60, Math.max(1, rampSeconds));
    const base = Math.min(max, Math.max(1, spikeBase));
    const atSeconds = Math.min(60, Math.max(0, spikeAt));
    const forSeconds = Math.min(60, Math.max(1, spikeFor));
    const total = Math.min(2000, Math.max(1, loadTotal));
    const profile: api.LoadProfile =
      profileKind === "fixed"
        ? { kind: "fixed", concurrency: max }
        : profileKind === "ramp_up"
          ? { kind: "ramp_up", start, max, ramp_ms: seconds * 1000 }
          : { kind: "spike", base, spike: max, at_ms: atSeconds * 1000, for_ms: forSeconds * 1000 };
    setConcurrency(max);
    setRampStart(start);
    setRampSeconds(seconds);
    setSpikeBase(base);
    setSpikeAt(atSeconds);
    setSpikeFor(forSeconds);
    setLoadTotal(total);
    releaseFocus();
    setLoading(true);
    setStatus("");
    const tabKey = active.key;
    const roomBase = roomBaseUrl.trim();
    try {
      if (!currentRoom || currentRoom.base_url !== roomBase) {
        await api.setRoomBaseUrl(currentRoomId, roomBase);
        setRooms((prev) =>
          prev.map((room) => (room.id === currentRoomId ? { ...room, base_url: roomBase } : room)),
        );
      }
      const report = await api.runLoad({
        baseUrl: roomBase,
        total,
        profile,
        draft: active.draft,
      });
      setTabs((prev) =>
        prev.map((tab) => (tab.key === tabKey ? { ...tab, loadReport: report } : tab)),
      );
      setResponseTab("load");
    } catch (err) {
      setStatus(String(err));
    } finally {
      setLoading(false);
      keepWindowInFront();
    }
  };

  const send = async () => {
    if (!currentRoomId || !active || sending || loading) return;
    setSending(true);
    setStatus("");
    const base = roomBaseUrl.trim();
    const tabKey = active.key;
    try {
      if (!currentRoom || currentRoom.base_url !== base) {
        await api.setRoomBaseUrl(currentRoomId, base);
        setRooms((prev) =>
          prev.map((room) => (room.id === currentRoomId ? { ...room, base_url: base } : room)),
        );
      }
      const result = await api.sendRequest({
        roomId: currentRoomId,
        selectedId: active.savedId,
        name: active.name,
        baseUrl: base,
        draft: active.draft,
      });
      setResponseTab("body");
      setTabs((prev) =>
        prev.map((tab) =>
          tab.key === tabKey
            ? {
                ...tab,
                savedId: result.saved.id,
                name: result.saved.name,
                draft: result.saved.draft,
                response: result.snapshot,
              }
            : tab,
        ),
      );
      setRequests((prev) => {
        const exists = prev.some((saved) => saved.id === result.saved.id);
        if (exists) return prev.map((saved) => (saved.id === result.saved.id ? result.saved : saved));
        return [result.saved, ...prev];
      });
    } catch (err) {
      setStatus(String(err));
    } finally {
      setSending(false);
    }
  };

  const draft = active?.draft ?? defaultDraft();
  const response = active?.response ?? null;
  const loadReport = active?.loadReport ?? null;
  const { path, params } = splitUrl(draft.url);
  const showBase = Boolean(roomBaseUrl.trim()) && !draft.url.includes("://");
  const statusClass =
    response && response.status >= 200 && response.status < 300 ? "status-ok" : "status-err";

  const writeParams = (pathValue: string, nextParams: QueryParam[]) => {
    setDraft({ ...draft, url: joinUrl(pathValue, nextParams) });
  };

  if (booting) {
    return (
      <div className="boot">
        <p>Loading…</p>
      </div>
    );
  }

  if (bootError) {
    return (
      <div className="boot">
        <h2>Could not reach the backend</h2>
        <p>{bootError}</p>
      </div>
    );
  }

  return (
    <div className="app-shell">
      <div className="app-main">
        <aside className="sidebar">
          <div className="side-title">
            <span>Collections</span>
            <span className="side-actions">
              <button
                type="button"
                className="icon-btn"
                title="New collection"
                onClick={() => setCreatingRoom((open) => !open)}
              >
                +
              </button>
            </span>
          </div>
          <input
            className="side-search"
            placeholder="Search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          {creatingRoom && (
            <input
              className="ghost-input"
              autoFocus
              placeholder="Collection name"
              value={roomNameDraft}
              onChange={(e) => setRoomNameDraft(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") void addRoom();
                if (e.key === "Escape") {
                  setCreatingRoom(false);
                  setRoomNameDraft("");
                }
              }}
            />
          )}

          <div className="tree">
            {visibleRooms.map((room) => {
              const open = openRooms.includes(room.id) && room.id === currentRoomId;
              return (
                <div className="collection" key={room.id}>
                  <div className={`tree-row ${room.id === currentRoomId ? "active" : ""}`}>
                    <button type="button" className="tree-main" onClick={() => void selectRoom(room)}>
                      <span className="chevron">{open ? "▾" : "▸"}</span>
                      <span className="name">{room.name || "Untitled"}</span>
                    </button>
                    <button
                      type="button"
                      className="icon-btn danger"
                      title="Delete collection"
                      onClick={() => void removeRoom(room.id)}
                    >
                      ×
                    </button>
                  </div>
                  {open && (
                    <div className="requests">
                      {visibleRequests.map((saved) => (
                        <div
                          key={saved.id}
                          className={`tree-row request ${active?.savedId === saved.id ? "active" : ""}`}
                        >
                          <button
                            type="button"
                            className="tree-main"
                            onClick={() => openRequest(saved)}
                          >
                            <span className={`method method-${saved.draft.method}`}>
                              {saved.draft.method.toUpperCase()}
                            </span>
                            <span className="name">{requestLabel(saved)}</span>
                          </button>
                          <button
                            type="button"
                            className="icon-btn danger"
                            title="Delete request"
                            onClick={() => void removeRequest(saved.id)}
                          >
                            ×
                          </button>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
            {visibleRooms.length === 0 && <p className="empty">Nothing found</p>}
          </div>
        </aside>

        <section className="workspace">
          <div className="crumb">
            <span className="crumb-name">{currentRoom?.name || "No collection"}</span>
            <label className="base-line">
              <span>baseUrl</span>
              <input
                placeholder="optional, e.g. https://api.example.com"
                value={roomBaseUrl}
                disabled={!currentRoomId}
                onChange={(e) => setRoomBaseUrl(e.target.value)}
                onBlur={() => void saveBaseUrl()}
                onKeyDown={(e) => {
                  if (e.key === "Enter") e.currentTarget.blur();
                }}
              />
            </label>
          </div>

          <div className="browser-tabs">
            {roomTabs.map((tab) => (
              <div key={tab.key} className={`browser-tab ${tab.key === activeKey ? "active" : ""}`}>
                <div className="tab-main" onClick={() => setActiveKey(tab.key)}>
                  <span className={`method method-${tab.draft.method}`}>
                    {tab.draft.method.toUpperCase()}
                  </span>
                  {tab.key === activeKey ? (
                    <input
                      className="tab-name"
                      value={tab.name}
                      placeholder="Untitled"
                      onChange={(e) =>
                        updateActive((current) => ({ ...current, name: e.target.value }))
                      }
                    />
                  ) : (
                    <span className="name">{tabLabel(tab)}</span>
                  )}
                </div>
                <button type="button" className="tab-close" onClick={() => closeTab(tab.key)}>
                  ×
                </button>
              </div>
            ))}
            <button
              type="button"
              className="tab-add"
              title="New request"
              disabled={!currentRoomId}
              onClick={newRequest}
            >
              +
            </button>
          </div>

          <div className="request-bar">
            <div className="urlbar">
              <select
                className={`method-select method-${draft.method}`}
                value={draft.method}
                disabled={!active}
                onChange={(e) => onMethodChange(e.target.value as HttpMethod)}
              >
                {METHODS.map((method) => (
                  <option key={method} value={method}>
                    {method.toUpperCase()}
                  </option>
                ))}
              </select>
              {showBase && <span className="base-chip">{`{{baseUrl}}`}</span>}
              <input
                className="url-input"
                placeholder="/path"
                value={draft.url}
                disabled={!active}
                onChange={(e) => setDraft({ ...draft, url: e.target.value })}
                onKeyDown={(e) => {
                  if (e.key === "Enter") void send();
                }}
              />
              <button
                type="button"
                className="load-run"
                disabled={!active || sending || loading}
                onClick={() => void runLoad()}
              >
                {loading ? "Running" : "Run"}
              </button>
              <button type="button" className="send" disabled={!active || sending || loading} onClick={() => void send()}>
                {sending ? "Sending" : "Send"}
              </button>
            </div>
            <div className="loadbar">
              <span className="load-kicker">Load test</span>
              <select
                value={profileKind}
                disabled={!active || loading}
                onChange={(e) => setProfileKind(e.target.value as "fixed" | "ramp_up" | "spike")}
              >
                <option value="fixed">Fixed</option>
                <option value="ramp_up">Ramp up</option>
                <option value="spike">Spike</option>
              </select>
              {profileKind === "ramp_up" && (
                <label>
                  <span>Start</span>
                  <input
                    type="number"
                    min={1}
                    max={50}
                    value={rampStart}
                    disabled={!active || loading}
                    onChange={(e) => setRampStart(Number(e.target.value))}
                  />
                </label>
              )}
              {profileKind === "spike" && (
                <label>
                  <span>Base</span>
                  <input
                    type="number"
                    min={1}
                    max={50}
                    value={spikeBase}
                    disabled={!active || loading}
                    onChange={(e) => setSpikeBase(Number(e.target.value))}
                  />
                </label>
              )}
              <label>
                <span>
                  {profileKind === "fixed" ? "Concurrent" : profileKind === "ramp_up" ? "Max" : "Spike"}
                </span>
                <input
                  type="number"
                  min={1}
                  max={50}
                  value={concurrency}
                  disabled={!active || loading}
                  onChange={(e) => setConcurrency(Number(e.target.value))}
                />
              </label>
              {profileKind === "ramp_up" && (
                <label>
                  <span>Ramp, s</span>
                  <input
                    type="number"
                    min={1}
                    max={60}
                    value={rampSeconds}
                    disabled={!active || loading}
                    onChange={(e) => setRampSeconds(Number(e.target.value))}
                  />
                </label>
              )}
              {profileKind === "spike" && (
                <>
                  <label>
                    <span>At, s</span>
                    <input
                      type="number"
                      min={0}
                      max={60}
                      value={spikeAt}
                      disabled={!active || loading}
                      onChange={(e) => setSpikeAt(Number(e.target.value))}
                    />
                  </label>
                  <label>
                    <span>For, s</span>
                    <input
                      type="number"
                      min={1}
                      max={60}
                      value={spikeFor}
                      disabled={!active || loading}
                      onChange={(e) => setSpikeFor(Number(e.target.value))}
                    />
                  </label>
                </>
              )}
              <label>
                <span>Requests</span>
                <input
                  type="number"
                  min={1}
                  max={2000}
                  value={loadTotal}
                  disabled={!active || loading}
                  onChange={(e) => setLoadTotal(Number(e.target.value))}
                />
              </label>
            </div>
            {status && <div className="status-note">{status}</div>}
          </div>

          <div className="work-split">
            <div className="request-pane">
              <div className="tabs">
                <button
                  type="button"
                  className={editorTab === "params" ? "active" : ""}
                  onClick={() => setEditorTab("params")}
                >
                  Params{params.length ? ` ${params.length}` : ""}
                </button>
                <button
                  type="button"
                  className={editorTab === "body" ? "active" : ""}
                  onClick={() => setEditorTab("body")}
                >
                  Body
                </button>
                <button
                  type="button"
                  className={editorTab === "headers" ? "active" : ""}
                  onClick={() => setEditorTab("headers")}
                >
                  Headers{draft.headers.length ? ` ${draft.headers.length}` : ""}
                </button>
              </div>

              <div className="editor">
                {!active && <p className="empty">Open a request or press +</p>}
                {active && editorTab === "params" && (
                  <>
                    <p className="pane-hint">Query parameters stay in sync with the URL.</p>
                    <KvTable
                      rows={params.map((param): Pair => [param.name, param.value])}
                      onChange={(rows) =>
                        writeParams(
                          path,
                          rows.map(([name, value]) => ({ name, value })),
                        )
                      }
                    />
                  </>
                )}
                {active && editorTab === "body" && (
                  <textarea
                    value={draft.body}
                    spellCheck={false}
                    onChange={(e) => setDraft({ ...draft, body: e.target.value })}
                  />
                )}
                {active && editorTab === "headers" && (
                  <>
                    <p className="pane-hint">Start typing a key to see common headers.</p>
                    <KvTable
                      rows={draft.headers}
                      keyList="common-headers"
                      valueList="common-values"
                      onChange={(headers) => setDraft({ ...draft, headers })}
                    />
                    <datalist id="common-headers">
                      {COMMON_HEADERS.map((name) => (
                        <option key={name} value={name} />
                      ))}
                    </datalist>
                    <datalist id="common-values">
                      {COMMON_VALUES.map((value) => (
                        <option key={value} value={value} />
                      ))}
                    </datalist>
                  </>
                )}
              </div>
            </div>

            <aside className="response">
              <div className="response-head">
                <div className="tabs plain">
                  <button
                    type="button"
                    className={responseTab === "body" ? "active" : ""}
                    onClick={() => setResponseTab("body")}
                  >
                    Response
                  </button>
                  <button
                    type="button"
                    className={responseTab === "headers" ? "active" : ""}
                    onClick={() => setResponseTab("headers")}
                  >
                    Headers{response ? ` ${response.headers.length}` : ""}
                  </button>
                  <button
                    type="button"
                    className={responseTab === "load" ? "active" : ""}
                    onClick={() => setResponseTab("load")}
                  >
                    Load
                  </button>
                </div>
                {responseTab !== "load" && response && (
                  <div className="spacer">
                    <span className={statusClass}>{statusText(response.status)}</span>
                    <span>{formatElapsed(response.elapsed_ms)}</span>
                    <span>{formatSize(response.body)}</span>
                  </div>
                )}
                {responseTab === "load" && loadReport && (
                  <div className="spacer">
                    <span className="status-ok">{loadReport.counted_ok} OK</span>
                    <span className={loadReport.counted_failed > 0 ? "status-err" : "status-ok"}>
                      {loadReport.counted_failed} failed
                    </span>
                    <span>{loadReport.rps.toFixed(1)} req/s</span>
                    <span>{formatElapsed(loadReport.elapsed_ms)}</span>
                  </div>
                )}
              </div>
              {responseTab === "body" && !response && (
                <p className="empty">Send a request to see the response</p>
              )}
              {responseTab === "body" && response && <CodeView text={response.body} />}
              {responseTab === "headers" && !response && (
                <p className="empty">Send a request to see the response</p>
              )}
              {responseTab === "headers" && response && (
                <div className="header-list">
                  {response.headers.length === 0 && <p className="empty">No headers</p>}
                  {response.headers.map(([name, value], index) => (
                    <div key={`${name}-${index}`}>
                      <span className="name">{name}</span>
                      <span>{value}</span>
                    </div>
                  ))}
                </div>
              )}
              {responseTab === "load" && !loadReport && (
                <p className="empty">Run a load test to see the report</p>
              )}
              {responseTab === "load" && loadReport && <CodeView text={reportJson(loadReport)} />}
            </aside>
          </div>
        </section>
      </div>
    </div>
  );
}

function KvTable({
  rows,
  onChange,
  keyList,
  valueList,
}: {
  rows: Pair[];
  onChange: (rows: Pair[]) => void;
  keyList?: string;
  valueList?: string;
}) {
  const edit = (index: number, column: 0 | 1, text: string) => {
    const next = rows.map((row): Pair => [row[0], row[1]]);
    if (index === rows.length) next.push(["", ""]);
    next[index][column] = text;
    onChange(next);
  };

  return (
    <div className="kv-table">
      <div className="kv-row kv-head">
        <span>Key</span>
        <span>Value</span>
        <span />
      </div>
      {[...rows, ["", ""] as Pair].map(([key, value], index) => {
        const isNew = index === rows.length;
        return (
          <div className={`kv-row ${isNew ? "kv-new" : ""}`} key={index}>
            <input
              placeholder={isNew ? "Add key" : "Key"}
              value={key}
              list={keyList}
              spellCheck={false}
              onChange={(e) => edit(index, 0, e.target.value)}
            />
            <input
              placeholder={isNew ? "Value" : ""}
              value={value}
              list={valueList}
              spellCheck={false}
              onChange={(e) => edit(index, 1, e.target.value)}
            />
            {isNew ? (
              <span />
            ) : (
              <button
                type="button"
                className="row-x"
                title="Remove"
                onClick={() => onChange(rows.filter((_, i) => i !== index))}
              >
                ×
              </button>
            )}
          </div>
        );
      })}
    </div>
  );
}

function CodeView({ text }: { text: string }) {
  const lines = text.length ? text.split("\n") : [""];
  const looksJson = text.trim().startsWith("{") || text.trim().startsWith("[");
  return (
    <div className="code">
      {lines.map((line, index) => (
        <div className="code-line" key={index}>
          <span className="ln">{index + 1}</span>
          {looksJson ? <JsonLine line={line} /> : <span className="src">{line || " "}</span>}
        </div>
      ))}
    </div>
  );
}

function JsonLine({ line }: { line: string }) {
  const re =
    /("(?:\\.|[^"\\])*")(\s*:)?|\b(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)\b|\b(true|false|null)\b/g;
  const parts: { text: string; cls?: string }[] = [];
  let last = 0;
  for (const match of line.matchAll(re)) {
    const index = match.index ?? 0;
    if (index > last) parts.push({ text: line.slice(last, index) });
    if (match[1] && match[2]) parts.push({ text: match[1], cls: "tok-key" }, { text: match[2] });
    else if (match[1]) parts.push({ text: match[1], cls: "tok-str" });
    else if (match[3]) parts.push({ text: match[3], cls: "tok-num" });
    else if (match[4]) parts.push({ text: match[4], cls: "tok-bool" });
    last = index + match[0].length;
  }
  if (last < line.length) parts.push({ text: line.slice(last) });
  if (parts.length === 0) parts.push({ text: " " });
  return (
    <span className="src">
      {parts.map((part, index) =>
        part.cls ? (
          <span key={index} className={part.cls}>
            {part.text}
          </span>
        ) : (
          <span key={index}>{part.text}</span>
        ),
      )}
    </span>
  );
}
