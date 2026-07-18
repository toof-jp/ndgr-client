import { useEffect, useRef, useState } from "react";
import { connect, type Connection, type NdgrMessage } from "./ndgr.ts";

const MAX_COMMENTS = 1000;

interface CommentEntry {
  id: number;
  message: NdgrMessage;
}

let nextId = 0;

function CommentRow({ message }: { message: NdgrMessage }) {
  const time = message.at != null ? new Date(message.at * 1000).toLocaleTimeString("ja-JP") : "";

  let user = "";
  let body: string;
  let premium = false;
  switch (message.type) {
    case "chat":
      user = message.name ?? message.rawUserId?.toString() ?? message.hashedUserId ?? "";
      premium = message.premium;
      body = message.content;
      break;
    case "gift":
      body = `🎁 ${message.advertiserName} さんが「${message.itemName}」を贈りました (${String(message.point)}pt)`;
      break;
    case "nicoad":
      body = `📣 ${message.content}`;
      break;
    case "notification":
      body = message.content;
      break;
  }

  return (
    <div className={`comment ${message.type}`}>
      <time>{time}</time>
      {message.type === "chat" && (
        <span className={premium ? "user premium" : "user"} title={user}>
          {user}
        </span>
      )}
      <span className="body">{body}</span>
    </div>
  );
}

export default function App() {
  const [url, setUrl] = useState("");
  const [proxy, setProxy] = useState("");
  const [connected, setConnected] = useState(false);
  const [status, setStatus] = useState("");
  const [isError, setIsError] = useState(false);
  const [comments, setComments] = useState<CommentEntry[]>([]);

  const connectionRef = useRef<Connection | null>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const stickToBottomRef = useRef(true);

  useEffect(() => {
    const list = listRef.current;
    if (list && stickToBottomRef.current) {
      list.scrollTop = list.scrollHeight;
    }
  }, [comments]);

  useEffect(() => () => connectionRef.current?.disconnect(), []);

  const showStatus = (text: string, error = false) => {
    setStatus(text);
    setIsError(error);
  };

  const disconnect = (text = "切断しました") => {
    connectionRef.current?.disconnect();
    connectionRef.current = null;
    setConnected(false);
    showStatus(text);
  };

  const handleConnect = async () => {
    const programUrl = url.trim();
    if (!programUrl) {
      showStatus("番組URLを入力してください", true);
      return;
    }

    setConnected(true);
    setComments([]);
    stickToBottomRef.current = true;

    try {
      connectionRef.current = await connect(programUrl, proxy.trim(), {
        onMessage: (message) => {
          setComments((prev) => {
            const next = [...prev, { id: nextId++, message }];
            return next.length > MAX_COMMENTS ? next.slice(next.length - MAX_COMMENTS) : next;
          });
        },
        onStatus: (text) => showStatus(text),
        onError: (text) => {
          showStatus(text, true);
          connectionRef.current = null;
          setConnected(false);
        },
      });
    } catch (e) {
      showStatus(
        `番組情報の取得に失敗しました: ${String(e)} (CORS の場合はプロキシを指定してください)`,
        true,
      );
      setConnected(false);
    }
  };

  const handleScroll = () => {
    const list = listRef.current;
    if (list) {
      stickToBottomRef.current = list.scrollHeight - list.scrollTop - list.clientHeight < 30;
    }
  };

  return (
    <>
      <header>
        <h1>NDGR Comment Viewer</h1>
        <input
          id="url"
          type="url"
          placeholder="https://live.nicovideo.jp/watch/lv…"
          spellCheck={false}
          value={url}
          disabled={connected}
          onChange={(e) => setUrl(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !connected) void handleConnect();
          }}
        />
        <input
          id="proxy"
          type="text"
          placeholder="CORSプロキシ (任意, URLの前に連結)"
          spellCheck={false}
          value={proxy}
          disabled={connected}
          onChange={(e) => setProxy(e.target.value)}
        />
        <button
          onClick={() => {
            if (connected) {
              disconnect();
            } else {
              void handleConnect();
            }
          }}
        >
          {connected ? "切断" : "接続"}
        </button>
      </header>
      <div id="status" className={isError ? "error" : ""}>
        {status}
      </div>
      <div id="comments" ref={listRef} onScroll={handleScroll}>
        {comments.map((entry) => (
          <CommentRow key={entry.id} message={entry.message} />
        ))}
      </div>
    </>
  );
}
