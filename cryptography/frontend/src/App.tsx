import { useEffect, useRef, useState } from "react";
import { Glyph } from "./assets/icons";
import { GROUP_SIZE, useGameSocket } from "./useGameSocket";
import { VictoryMessage } from "./victoryMessage";

const MID = GROUP_SIZE / 2;
const STEP = 360 / 26;

const ALPHA_NUMERIC = Array.from({ length: 26 }, (_, i) =>
  String.fromCharCode(65 + i),
);

function shortestDiff(from: number, to: number) {
  let diff = to - from;
  if (diff > ALPHA_NUMERIC.length / 2) diff -= ALPHA_NUMERIC.length;
  if (diff < -ALPHA_NUMERIC.length / 2) diff += ALPHA_NUMERIC.length;
  return diff;
}

function App() {
  const game = useGameSocket();

  const [rotationGroup, setRotationGroup] = useState<number[]>(
    new Array(GROUP_SIZE).fill(0),
  );
  const shownIndex = useRef<number[]>(new Array(GROUP_SIZE).fill(0));

  const [glitch, setGlitch] = useState(false);
  const prevCoded = useRef("");

  useEffect(() => {
    const deltas = game.rings.map((target, i) => {
      const current = shownIndex.current[i];
      if (current === target) return 0;
      shownIndex.current[i] = target;
      return STEP * shortestDiff(current, target);
    });
    if (deltas.every((d) => d === 0)) return;
    setRotationGroup((prev) => prev.map((deg, i) => deg + deltas[i]));
  }, [game.rings]);

  useEffect(() => {
    const changed = prevCoded.current !== "" && game.coded !== prevCoded.current;
    prevCoded.current = game.coded;
    if (!changed) return;
    setGlitch(true);
    const t = window.setTimeout(() => setGlitch(false), 700);
    return () => window.clearTimeout(t);
  }, [game.coded]);

  const handleClick = (ringIndex: number, charIndex: number) => {
    if (game.solved) return;
    game.rotate(ringIndex, charIndex);
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

  const ringGroup = (start: number, end?: number) => {
    return (
      <>
        {game.rings.slice(start, end).map((_, ringIndex) => (
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
    <main className={`cryptex${game.solved ? " solved" : ""}`}>
      <div className={`hologram${glitch ? " glitch" : ""}`} aria-live="polite">
        <span className="hologram-text" data-text={game.coded}>
          {game.connected ? game.coded || "········" : "········"}
        </span>
        <span className="hologram-base" />
      </div>

      {game.solved && game.values && (
        <section className="victory-message" role="status">
          <VictoryMessage values={game.values} />
        </section>
      )}

      <ul className="ring-group">
        {ringGroup(0, MID)}
        <div className="split">
          <div
            className={`submit${game.solved ? " disabled" : ""}`}
            onClick={() => !game.solved && game.attempt()}
            role="button"
            aria-disabled={game.solved}
            title={game.solved ? "Opened" : "Submit"}
          >
            <Glyph />
          </div>
        </div>
        {ringGroup(MID)}
      </ul>

      <footer className="status">
        <span className={`dot ${game.connected ? "on" : "off"}`} />
        {game.attempts > 0 && <span className="attempts">· {game.attempts} attempts</span>}
        {game.error && <span className="err">· {game.error}</span>}
      </footer>
    </main>
  );
}

export default App;
