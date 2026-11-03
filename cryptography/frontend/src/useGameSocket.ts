import { useCallback, useEffect, useRef, useState } from "react";

export const GROUP_SIZE = 8;

export interface SolvedValues {
  password: string;
  coded: string;
  attempts: number;
  solvedAt: string;
}

type ServerMsg =
  | {
      type: "state";
      coded: string;
      rings: number[];
      attempts: number;
      solved: boolean;
      values?: SolvedValues;
    }
  | { type: "solved"; values: SolvedValues }
  | { type: "error"; message: string };

type ClientMsg =
  | { type: "rotate"; ring: number; index: number }
  | { type: "attempt" }
  | { type: "reset"; token: string };

export interface GameSocket {
  connected: boolean;
  coded: string;
  rings: number[];
  attempts: number;
  solved: boolean;
  values: SolvedValues | null;
  error: string | null;
  rotate: (ring: number, index: number) => void;
  attempt: () => void;
  reset: (token: string) => void;
}

function resolveUrl(): string {
  const configured = import.meta.env.VITE_WS_URL as string | undefined;
  if (configured) return configured;
  const proto = window.location.protocol === "https:" ? "wss" : "ws";
  return `${proto}://${window.location.host}/ws`;
}

export function useGameSocket(): GameSocket {
  const [connected, setConnected] = useState(false);
  const [coded, setCoded] = useState("");
  const [rings, setRings] = useState<number[]>(new Array(GROUP_SIZE).fill(0));
  const [attempts, setAttempts] = useState(0);
  const [solved, setSolved] = useState(false);
  const [values, setValues] = useState<SolvedValues | null>(null);
  const [error, setError] = useState<string | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const retryRef = useRef(0);
  const timerRef = useRef<number | null>(null);

  useEffect(() => {
    const url = resolveUrl();
    let disposed = false;

    const connect = () => {
      if (disposed) return;
      const socket = new WebSocket(url);
      wsRef.current = socket;
      const isLive = () => !disposed && wsRef.current === socket;

      socket.onopen = () => {
        if (!isLive()) return;
        retryRef.current = 0;
        setConnected(true);
        setError(null);
      };

      socket.onmessage = (event) => {
        if (!isLive()) return;
        let msg: ServerMsg;
        try {
          msg = JSON.parse(event.data);
        } catch {
          console.warn("unparseable frame", event.data);
          return;
        }
        switch (msg.type) {
          case "state":
            setCoded(msg.coded);
            setRings(msg.rings);
            setAttempts(msg.attempts);
            setSolved(msg.solved);
            if (msg.values) setValues(msg.values);
            if (!msg.solved) setValues(null);
            break;
          case "solved":
            setSolved(true);
            setValues(msg.values);
            break;
          case "error":
            setError(msg.message);
            break;
        }
      };

      socket.onclose = () => {
        if (!isLive()) return;
        setConnected(false);
        const delay = Math.min(1000 * 2 ** retryRef.current, 15000);
        retryRef.current += 1;
        timerRef.current = window.setTimeout(connect, delay);
      };
    };

    connect();

    return () => {
      disposed = true;
      if (timerRef.current) window.clearTimeout(timerRef.current);
      const socket = wsRef.current;
      wsRef.current = null;
      if (!socket) return;
      if (socket.readyState === WebSocket.CONNECTING) {
        socket.onopen = () => socket.close();
      } else {
        socket.close();
      }
    };
  }, []);

  const send = useCallback((msg: ClientMsg) => {
    const ws = wsRef.current;
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(msg));
    }
  }, []);

  const rotate = useCallback(
    (ring: number, index: number) => {
      setRings((prev) => {
        if (prev[ring] === index) return prev;
        const next = [...prev];
        next[ring] = index;
        return next;
      });
      send({ type: "rotate", ring, index });
    },
    [send],
  );

  const attempt = useCallback(() => send({ type: "attempt" }), [send]);
  const reset = useCallback(
    (token: string) => send({ type: "reset", token }),
    [send],
  );

  return {
    connected,
    coded,
    rings,
    attempts,
    solved,
    values,
    error,
    rotate,
    attempt,
    reset,
  };
}
