import init, { fetch_web_socket_url, stream_comments } from "../wasm/pkg/ndgr_client_wasm.js";
import wasmUrl from "../wasm/pkg/ndgr_client_wasm_bg.wasm?url";

export type NdgrMessage =
  | {
      type: "chat";
      at: number | null;
      content: string;
      name: string | null;
      rawUserId: number | null;
      hashedUserId: string | null;
      premium: boolean;
    }
  | {
      type: "gift";
      at: number | null;
      advertiserName: string;
      itemName: string;
      point: number;
      message: string;
    }
  | { type: "nicoad"; at: number | null; content: string }
  | { type: "notification"; at: number | null; kind: string; content: string };

export interface ConnectionCallbacks {
  onMessage: (message: NdgrMessage) => void;
  onStatus: (status: string) => void;
  onError: (error: string) => void;
}

export interface Connection {
  disconnect: () => void;
}

let wasmReady: Promise<unknown> | null = null;

function ensureWasm(): Promise<unknown> {
  wasmReady ??= init({ module_or_path: wasmUrl });
  return wasmReady;
}

export async function connect(
  programUrl: string,
  proxyPrefix: string,
  callbacks: ConnectionCallbacks,
): Promise<Connection> {
  await ensureWasm();

  callbacks.onStatus("番組情報を取得中…");
  const webSocketUrl: string = await fetch_web_socket_url(programUrl, proxyPrefix);

  let alive = true;
  let keepSeatTimer: ReturnType<typeof setInterval> | null = null;

  const ws = new WebSocket(webSocketUrl);

  const disconnect = () => {
    alive = false;
    if (keepSeatTimer !== null) clearInterval(keepSeatTimer);
    keepSeatTimer = null;
    ws.close();
  };

  ws.onopen = () => {
    callbacks.onStatus("WebSocket 接続完了、視聴開始中…");
    ws.send(JSON.stringify({ type: "startWatching", data: { reconnect: false } }));
  };

  ws.onmessage = (event) => {
    const message = JSON.parse(event.data as string) as {
      type: string;
      data?: Record<string, unknown>;
    };
    switch (message.type) {
      case "messageServer": {
        const viewUri = message.data?.["viewUri"] as string;
        callbacks.onStatus("コメント受信中");
        stream_comments(viewUri, proxyPrefix, (json: string) => {
          if (!alive) return false;
          callbacks.onMessage(JSON.parse(json) as NdgrMessage);
          return true;
        }).catch((e: unknown) => {
          if (alive) {
            callbacks.onError(`ストリームエラー: ${String(e)}`);
            disconnect();
          }
        });
        break;
      }
      case "seat": {
        const intervalSec = message.data?.["keepIntervalSec"] as number;
        if (keepSeatTimer !== null) clearInterval(keepSeatTimer);
        keepSeatTimer = setInterval(() => {
          if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: "keepSeat" }));
          }
        }, intervalSec * 1000);
        break;
      }
      case "ping":
        ws.send(JSON.stringify({ type: "pong" }));
        break;
      case "disconnect":
        callbacks.onError(`切断されました: ${String(message.data?.["reason"] ?? "")}`);
        disconnect();
        break;
    }
  };

  ws.onerror = () => {
    if (alive) callbacks.onError("WebSocket エラー");
  };

  return { disconnect };
}
