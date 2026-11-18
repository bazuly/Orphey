import { useCallback, useEffect, useMemo, useState } from "react";
import * as api from "./api";
import type { HttpMethod, RequestDraft, ResponseSnapshot, Room, SavedRequest } from "./types";

const METHODS: HttpMethod[] = ["get", "post", "put", "patch", "delete"];

const HEADER_TEMPLATES: [string, string][] = [
  ["Content-Type", "application/json"],
  ["Accept", "application/json"],
  ["Authorization", "Bearer "],
  ["User-Agent", "Orphey"],
];

const defaultDraft = (): RequestDraft => ({
  method: "get",
  url: "",
  headers: [],
  body: '{\n  "id": 1\n}',
});

// TODO причесать тут все
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

function requestLabel(saved: SavedRequest): string {
  return saved.name.trim() || saved.draft.url.trim() || "Untitled";
}

export default function App() {
  const [rooms, setRooms] = useState<Room[]>([]);
  const [requests, setRequests] = useState<SavedRequest[]>([]);
  const [currentRoomId, setCurrentRoomId] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const [roomNameDraft, setRoomNameDraft] = useState("");
  const [creatingRoom, setCreatingRoom] = useState(false);
  const [query, setQuery] = useState("");
  const [roomBaseUrl, setRoomBaseUrl] = useState("");

  const [requestName, setRequestName] = useState("");
  const [draft, setDraft] = useState<RequestDraft>(defaultDraft);
  const [response, setResponse] = useState<ResponseSnapshot | null>(null);
  const [status, setStatus] = useState("");
  const [sending, setSending] = useState(false);
  const [booting, setBooting] = useState(true);
  const [bootError, setBootError] = useState<string | null>(null);
  const [editorTab, setEditorTab] = useState<"body" | "headers">("body");
  const [responseTab, setResponseTab] = useState<"body" | "headers">("body");

  const currentRoom = useMemo(
    () => rooms.find((r) => r.id === currentRoomId),
    [rooms, currentRoomId],
  );

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
    const room = data.rooms.find((r) => r.id === data.current_room_id);
    if (room) setRoomBaseUrl(room.base_url);
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

  const selectRoom = async (room: Room) => {
    if (room.id === currentRoomId) return;
    setCurrentRoomId(room.id);
    setRoomBaseUrl(room.base_url);
    setSelectedId(null);
    setRequestName("");
    setDraft(defaultDraft());
    setResponse(null);
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
      setRoomBaseUrl(room.base_url);
      setSelectedId(null);
      setRequestName("");
      setDraft(defaultDraft());
      setResponse(null);
      setRequests([]);
    } catch (err) {
      setStatus(String(err));
    }
  };

  const removeRoom = async (roomId: string) => {
    try {
      await api.deleteRoom(roomId);
      const next = rooms.filter((r) => r.id !== roomId);
      setRooms(next);
      if (currentRoomId === roomId) {
        const first = next[0];
        if (first) await selectRoom(first);
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
        prev.map((r) => (r.id === currentRoomId ? { ...r, base_url: base } : r)),
      );
      setRoomBaseUrl(base);
    } catch (err) {
      setStatus(String(err));
    }
  };

  const openRequest = (saved: SavedRequest) => {
    setSelectedId(saved.id);
    setRequestName(saved.name);
    setDraft(saved.draft);
    setResponse(saved.response);
  };

  const newRequest = () => {
    setSelectedId(null);
    setRequestName("");
    setDraft(defaultDraft());
    setResponse(null);
  };

  const removeRequest = async (id: string) => {
    try {
      await api.deleteRequest(id);
      setRequests((prev) => prev.filter((r) => r.id !== id));
      if (selectedId === id) newRequest();
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

  const addHeader = () => {
    setDraft((prev) => ({
      ...prev,
      headers: [...prev.headers, ["", ""]],
    }));
  };

  const addHeaderTemplate = (name: string, value: string) => {
    setDraft((prev) => {
      if (prev.headers.some(([n]) => n.toLowerCase() === name.toLowerCase())) {
        return prev;
      }
      return { ...prev, headers: [...prev.headers, [name, value]] };
    });
  };

  const send = async () => {
    if (!currentRoomId || sending) return;
    setSending(true);
    setStatus("");
    const base = roomBaseUrl.trim();
    try {
      if (!currentRoom || currentRoom.base_url !== base) {
        await api.setRoomBaseUrl(currentRoomId, base);
        setRooms((prev) =>
          prev.map((r) => (r.id === currentRoomId ? { ...r, base_url: base } : r)),
        );
      }
      const result = await api.sendRequest({
        roomId: currentRoomId,
        selectedId,
        name: requestName,
        baseUrl: base,
        draft,
      });
      setResponse(result.snapshot);
      setResponseTab("body");
      setSelectedId(result.saved.id);
      setRequestName(result.saved.name);
      setRequests((prev) => {
        const exists = prev.some((r) => r.id === result.saved.id);
        if (exists) {
          return prev.map((r) => (r.id === result.saved.id ? result.saved : r));
        }
        return [result.saved, ...prev];
      });
    } catch (err) {
      setStatus(String(err));
    } finally {
      setSending(false);
    }
  };

  const statusClass =
    response && response.status >= 200 && response.status < 300 ? "status-ok" : "status-err";

  if (booting) {
    return (
      <div className="boot">
        <p>Загрузка…</p>
      </div>
    );
  }

  if (bootError) {
    return (
      <div className="boot">
        <h2 style={{ marginTop: 0, fontWeight: 600 }}>Не удалось подключиться к backend</h2>
        <p>{bootError}</p>
      </div>
    );
  }

  return (
    <div className="app-shell">
      <header className="topbar">
        <span className="logo">Orphey</span>
        <div className="top-tab">
          <span className={`method method-${draft.method}`}>{draft.method.toUpperCase()}</span>
          <input
            placeholder="Untitled"
            value={requestName}
            onChange={(e) => setRequestName(e.target.value)}
          />
        </div>
        <span className="env">{currentRoom?.name}</span>
      </header>

      <div className="app-main">
        <aside className="sidebar">
          <div className="side-title">
            <span>Collections</span>
            <button
              type="button"
              className="icon-btn"
              title="New collection"
              onClick={() => setCreatingRoom((open) => !open)}
            >
              +
            </button>
          </div>
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
          <input
            className="side-search"
            placeholder="Search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />

          <div className="tree">
            {visibleRooms.map((room) => (
              <div className="collection" key={room.id}>
                <div className={`tree-row ${room.id === currentRoomId ? "active" : ""}`}>
                  <button type="button" className="tree-main" onClick={() => void selectRoom(room)}>
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

                {room.id === currentRoomId && (
                  <div className="requests">
                    <label className="base-line">
                      <span>base</span>
                      <input
                        placeholder="https://api.example.com"
                        value={roomBaseUrl}
                        onChange={(e) => setRoomBaseUrl(e.target.value)}
                        onBlur={() => void saveBaseUrl()}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") e.currentTarget.blur();
                        }}
                      />
                    </label>
                    {visibleRequests.map((saved) => (
                      <div
                        key={saved.id}
                        className={`tree-row ${saved.id === selectedId ? "active" : ""}`}
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
                    <button type="button" className="text-btn" onClick={newRequest}>
                      + Request
                    </button>
                  </div>
                )}
              </div>
            ))}
            {visibleRooms.length === 0 && <p className="empty">Nothing found</p>}
          </div>
        </aside>

        <section className="workspace">
          <div className="urlbar">
            <select
              className={`method-select method-${draft.method}`}
              value={draft.method}
              onChange={(e) => onMethodChange(e.target.value as HttpMethod)}
            >
              {METHODS.map((method) => (
                <option key={method} value={method}>
                  {method.toUpperCase()}
                </option>
              ))}
            </select>
            <input
              className="url-input"
              placeholder="/path or https://…"
              value={draft.url}
              onChange={(e) => setDraft({ ...draft, url: e.target.value })}
              onKeyDown={(e) => {
                if (e.key === "Enter") void send();
              }}
            />
            <button type="button" className="send" disabled={sending} onClick={() => void send()}>
              {sending ? "Sending" : "Send"}
            </button>
          </div>

          <div className="tabs">
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
            {editorTab === "body" ? (
              <textarea
                value={draft.body}
                spellCheck={false}
                onChange={(e) => setDraft({ ...draft, body: e.target.value })}
              />
            ) : (
              <>
                <div className="kv kv-head">
                  <span>Name</span>
                  <span>Value</span>
                  <span />
                </div>
                {draft.headers.map(([name, value], index) => (
                  <div className="kv" key={index}>
                    <input
                      placeholder="Name"
                      value={name}
                      onChange={(e) => {
                        const headers = [...draft.headers];
                        headers[index] = [e.target.value, value];
                        setDraft({ ...draft, headers });
                      }}
                    />
                    <input
                      placeholder="Value"
                      value={value}
                      onChange={(e) => {
                        const headers = [...draft.headers];
                        headers[index] = [name, e.target.value];
                        setDraft({ ...draft, headers });
                      }}
                    />
                    <button
                      type="button"
                      className="icon-btn danger"
                      onClick={() =>
                        setDraft({
                          ...draft,
                          headers: draft.headers.filter((_, i) => i !== index),
                        })
                      }
                    >
                      ×
                    </button>
                  </div>
                ))}
                <div className="header-tools">
                  <button type="button" className="text-btn" onClick={addHeader}>
                    + Add header
                  </button>
                  <select
                    className="flat-select"
                    defaultValue=""
                    onChange={(e) => {
                      const idx = Number(e.target.value);
                      if (idx >= 0) {
                        const [n, v] = HEADER_TEMPLATES[idx];
                        addHeaderTemplate(n, v);
                      }
                      e.target.value = "";
                    }}
                  >
                    <option value="">Template…</option>
                    {HEADER_TEMPLATES.map(([n], i) => (
                      <option key={n} value={i}>
                        {n}
                      </option>
                    ))}
                  </select>
                </div>
              </>
            )}
          </div>
          {status && <div className="status-note">{status}</div>}
        </section>

        <aside className="response">
          <div className="response-head">
            <div className="tabs" style={{ border: "none", padding: 0 }}>
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
                Headers
              </button>
            </div>
            {response && (
              <div className="spacer">
                <span className={statusClass}>{response.status}</span>
                <span>{response.elapsed_ms} ms</span>
              </div>
            )}
          </div>
          {!response && <p className="empty">Run a request to see the response</p>}
          {response && responseTab === "body" && <CodeView text={response.body} />}
          {response && responseTab === "headers" && (
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
        </aside>
      </div>
    </div>
  );
}

function CodeView({ text }: { text: string }) {
  const lines = text.length ? text.split("\n") : [""];
  return (
    <div className="code">
      {lines.map((line, index) => (
        <div className="code-line" key={index}>
          <span className="ln">{index + 1}</span>
          <span className="src">{line || " "}</span>
        </div>
      ))}
    </div>
  );
}
