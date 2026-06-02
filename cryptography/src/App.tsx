import { useMemo, useState } from "react";
import { Glyph } from "./assets/icons";

function App() {
  const groupSize = 8;
  const mid = groupSize / 2;
  const step = 360 / 26;

  const [input, setInput] = useState("");
  const [correct, setCorrect] = useState(false);
  const [locked, setLocked] = useState(false);

  const [rotationGroup, setRotationGroup] = useState<number[]>(
    new Array(groupSize).fill(0),
  );

  const [indexGroup, setIndexGroup] = useState<number[]>(
    new Array(groupSize).fill(0),
  );

  const alphaNumeric = Array.from({ length: 26 }, (_, i) =>
    String.fromCharCode(65 + i),
  );

  const currentValues = useMemo(() => {
    const newGroup = indexGroup.map((i) => alphaNumeric[i]);
    const concat = newGroup.join("");

    setCorrect(
      concat.toLowerCase() === input.toLowerCase() &&
        input.length === groupSize,
    );

    return newGroup;
  }, [indexGroup, input]);

  const handleClick = (ringIndex: number, charIndex: number) => {
    console.log(rotationGroup[ringIndex]);
    setLocked(false);

    const currentIndex = indexGroup[ringIndex];
    let diff = charIndex - currentIndex;

    if (diff > alphaNumeric.length / 2) diff -= alphaNumeric.length;
    if (diff < -alphaNumeric.length / 2) diff += alphaNumeric.length;

    setRotationGroup((prev) => {
      const next = [...prev];
      next[ringIndex] += step * diff;
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
      ((Math.round(rotation / step) % alphaNumeric.length) +
        alphaNumeric.length) %
      alphaNumeric.length;

    const distance = Math.min(
      Math.abs(charIndex - visibleIndex),
      alphaNumeric.length - Math.abs(charIndex - visibleIndex),
    );

    if (locked && visibleIndex === charIndex) return "victory";

    if (distance <= 1) {
      console.log(charIndex);
    }

    if (distance === 0) return "selected";
    if (distance <= 4) return "near";
    if (distance === 5) return "far";

    return "hidden";
  };

  const submit = () => {
    const concat = currentValues.join("");
    if (concat.toLowerCase() === input.toLowerCase()) {
      setLocked(true);
    }
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
              {alphaNumeric.map((char, charIndex) => (
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
      <ul className="ring-group">
        {ringGroup(0, mid)}
        <div className="split">
          <div
            className={correct ? "submit victory" : "submit"}
            onClick={submit}
          >
            <Glyph />
          </div>
        </div>
        {ringGroup(mid)}

      </ul>

      <div className="answer">{input}</div>

      <input
        type="text"
        maxLength={groupSize}
        onChange={(e) => setInput(e.currentTarget.value)}
      />
    </>
  );
}

export default App;
