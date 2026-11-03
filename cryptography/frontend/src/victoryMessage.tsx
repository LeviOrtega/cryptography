import type { SolvedValues } from "./useGameSocket";

export function VictoryMessage({ values }: { values: SolvedValues }) {
  const when = new Date(values.solvedAt).toLocaleString();

  return (
    <>
      <h1 className="victory-title">What is this?</h1>

      <p className="victory-body">
        Sensualist of spice mercantile, take heed of my plight: death feels
        better when earned.
      </p>

      <p className="victory-word">U-12</p>

      <p className="victory-meta">
        {when} - {values.attempts}{" "}
        {values.attempts == 1 ? "attempt" : "attempts"}
      </p>
    </>
  );
}
