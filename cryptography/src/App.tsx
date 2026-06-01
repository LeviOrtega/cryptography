import { useCallback, useEffect, useMemo, useState } from "react";
import { Glyph } from "./assets/icons";

function App() {
  const groupSize = 8;
  const mid = groupSize / 2;
  const step = 360 / 26;

  const [input, setInput] = useState<string>("");
  const [correct, setCorrect] = useState<boolean>(false);
  const [locked, setLocked] = useState<boolean>(false);
  const [diff, setDiff] = useState<number>(0);

  const [rotationGroup, setRotationGroup] = useState<number[]>(
    new Array(groupSize).fill(0),
  );
  const [indexGroup, setIndexGroup] = useState<number[]>(
    new Array(groupSize).fill(0),
  );
  const alphaNumeric: string[] = [
    ...Array.from({ length: 26 }, (_, i) => String.fromCharCode(65 + i)),
    //...Array.from({ length: 10 }, (_, i) => String.fromCharCode(48 + i)),
  ] as string[];

  const currentValues = useMemo(() => {
    const newGroup = indexGroup.map((index) => alphaNumeric[index]);
    const concat = newGroup.join("");
    if (
      concat.toLowerCase() === input.toLowerCase() &&
      input.length == groupSize
    ) {
      setCorrect(true);
    } else {
      setCorrect(false);
    }
    return newGroup;
  }, [indexGroup, input, setCorrect]);

  const submit = () => {
    const concat = currentValues.join("");
    if (concat.toLowerCase() === input.toLowerCase()) {
      setLocked(true);
    }
  };

  const handleClick = (ringIndex: number, charIndex: number) => {
    setLocked(false);
    const currentIndex = indexGroup[ringIndex];

    let diff = charIndex - currentIndex;

    if (diff > alphaNumeric.length / 2) diff -= alphaNumeric.length; // large positive jump
    if (diff < -alphaNumeric.length / 2) diff += alphaNumeric.length;

    setDiff(diff);

    setRotationGroup((prev) => {
      const next = [...prev];
      next[ringIndex] = next[ringIndex] + step * diff;
      return next;
    });

    setIndexGroup((prev) => {
      const next = [...prev];
      next[ringIndex] = charIndex;
      return next;
    });
  };

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement, HTMLInputElement>,
  ) => {
    setInput(e.currentTarget.value);
  };

  const getTransform = (ringIndex: number): string => {
    const transform = rotationGroup[ringIndex];
    return `rotateX(${-transform}deg)`;
  };

  return (
    <>
      <ul className="ring-group">
        {indexGroup.slice(0, mid).map((_, ringIndex) => (
          <li className="cryptex-container" key={ringIndex}>
            <div
              className="ring"
              style={
                {
                  "--d": Math.abs(diff) * 4,
                  "--s": `${1 / Math.abs(diff)}s`,
                  transform: getTransform(ringIndex),
                } as React.CSSProperties
              }
            >
              {alphaNumeric.map((char, charIndex) => (
                <div
                  key={char + charIndex}
                  className={
                    locked && rotationGroup[ringIndex] == charIndex
                      ? "letter victory"
                      : indexGroup[ringIndex] == charIndex
                        ? "letter selected"
                        : "letter"
                  }
                  style={{ "--i": charIndex } as React.CSSProperties}
                  onClick={() => handleClick(ringIndex, charIndex)}
                >
                  {char}
                </div>
              ))}
            </div>
          </li>
        ))}
        <div className="split">
          <div
            className={correct ? "submit victory" : "submit"}
            onClick={submit}
          >
            <Glyph />
          </div>
        </div>

        {indexGroup.slice(mid).map((_, ringIndex) => (
          <li className="cryptex-container" key={ringIndex + mid}>
            <div
              className="ring"
              style={
                {
                  "--d": Math.abs(diff) * 4,
                  "--s": `${1 / Math.abs(diff)}s`,
                  transform: getTransform(ringIndex + mid),
                } as React.CSSProperties
              }
            >
              {alphaNumeric.map((char, charIndex) => (
                <div
                  key={char + charIndex}
                  className={
                    locked && rotationGroup[ringIndex + mid] == charIndex
                      ? "letter victory"
                      : indexGroup[ringIndex + mid] == charIndex
                        ? "letter selected"
                        : "letter"
                  }
                  style={{ "--i": charIndex } as React.CSSProperties}
                  onClick={() => handleClick(ringIndex + mid, charIndex)}
                >
                  {char}
                </div>
              ))}
            </div>
          </li>
        ))}
      </ul>
      <div className="answer">{input}</div>
      <input
        type="text"
        maxLength={groupSize}
        minLength={groupSize}
        onChange={(e) => handleChange(e)}
      ></input>
    </>
  );
}

export default App;
