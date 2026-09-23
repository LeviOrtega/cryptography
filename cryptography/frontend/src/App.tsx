import { useEffect, useRef, useState } from "react";
import { Glyph } from "./assets/icons";

const GROUP_SIZE = 8;
const MID = GROUP_SIZE / 2;
const STEP = 360 / 26;
const WS_URL = "ws://127.0.0.1:8080";

const ALPHA_NUMERIC = Array.from({ length: 26 }, (_, i) =>
  String.fromCharCode(65 + i),
);

interface AttemptPayload {
  attempt: number[];
}

function App() {
  const [codedMessage, setCodedMessage] = useState("");

  const [rotationGroup, setRotationGroup] = useState<number[]>(
    new Array(GROUP_SIZE).fill(0),
  );

  const [indexGroup, setIndexGroup] = useState<number[]>(
    new Array(GROUP_SIZE).fill(0),
  );

  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    const socket = new WebSocket(WS_URL);
    wsRef.current = socket;

    socket.onopen = () => console.log("connected!");
    socket.onmessage = (event) => setCodedMessage(event.data);
    socket.onerror = (err) => console.error("error:", err);

    return () => socket.close();
  }, []);

  const handleClick = (ringIndex: number, charIndex: number) => {
    const currentIndex = indexGroup[ringIndex];
    let diff = charIndex - currentIndex;

    if (diff > ALPHA_NUMERIC.length / 2) diff -= ALPHA_NUMERIC.length;
    if (diff < -ALPHA_NUMERIC.length / 2) diff += ALPHA_NUMERIC.length;

    setRotationGroup((prev) => {
      const next = [...prev];
      next[ringIndex] += STEP * diff;
      return next;
    });

    setIndexGroup((prev) => {
      const next = [...prev];
      next[ringIndex] = charIndex;
      return next;
    });
  };

  const getTransform = (i: number) => `rotateX(${-rotationGroup[i]}deg)`;

  const getLetterState = (ringIndex: number, charIndex: number) => {
    const rotation = rotationGroup[ringIndex];

    const visibleIndex =
      ((Math.round(rotation / STEP) % ALPHA_NUMERIC.length) +
        ALPHA_NUMERIC.length) %
      ALPHA_NUMERIC.length;

    const distance = Math.min(
      Math.abs(charIndex - visibleIndex),
      ALPHA_NUMERIC.length - Math.abs(charIndex - visibleIndex),
    );

    if (distance === 0) return "selected";
    if (distance <= 4) return "near";
    if (distance === 5) return "far";

    return "hidden";
  };

  const submit = () => {
    const payload: AttemptPayload = { attempt: indexGroup };
    wsRef.current?.send(JSON.stringify(payload));
  };

  const ringGroup = (start: number, end?: number) => {
    return (
      <>
        {indexGroup.slice(start, end).map((_, ringIndex) => (
          <li className="cryptex-container" key={ringIndex + start}>
            <div
              className="ring"
              style={
                {
                  "--s": "0.6s",
                  "--d": "26",
                  transform: getTransform(ringIndex + start),
                } as React.CSSProperties
              }
            >
              {ALPHA_NUMERIC.map((char, charIndex) => (
                <div
                  key={char + charIndex}
                  className={`letter ${getLetterState(ringIndex + start, charIndex)}`}
                  style={{ "--i": charIndex } as React.CSSProperties}
                  onClick={() => handleClick(ringIndex + start, charIndex)}
                >
                  {char}
                </div>
              ))}
            </div>
          </li>
        ))}
      </>
    );
  };

  return (
    <>
      <div className="coded-message">{codedMessage}</div>
      <ul className="ring-group">
        {ringGroup(0, MID)}
        <div className="split">
          <div className="submit" onClick={submit}>
            <Glyph />
          </div>
        </div>
        {ringGroup(MID)}
      </ul>
    </>
  );
}

export default App;
