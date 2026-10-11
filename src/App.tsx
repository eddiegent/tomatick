import "./App.css";
import { engine, formatClock, useSnapshot } from "./engine/api";

// Development placeholder that proves the commands and events work end to
// end. The real tomato widget replaces it in milestone 3.
function App() {
  const snapshot = useSnapshot();

  if (!snapshot) {
    return <main className="placeholder" />;
  }

  const running = snapshot.state === "running";
  const heading = snapshot.phase ?? "countdown";

  return (
    <main className="placeholder">
      <p className="phase">{heading.replace("_", " ")}</p>
      <h1 className="clock">{formatClock(snapshot.remaining_ms)}</h1>
      <p className="detail">
        {snapshot.state.replace("_", " ")}
        {snapshot.label ? ` | ${snapshot.label}` : ""}
        {snapshot.segment
          ? ` | segment ${snapshot.segment.index + 1}/${snapshot.segment.count}`
          : ""}
      </p>
      <div className="controls">
        <button onClick={() => engine.toggle()}>
          {running ? "Pause" : "Start"}
        </button>
        <button onClick={() => engine.skip()} disabled={snapshot.mode !== "pomodoro"}>
          Skip
        </button>
        <button onClick={() => engine.reset()}>Reset</button>
        <button
          onClick={() =>
            engine.setMode(snapshot.mode === "pomodoro" ? "countdown" : "pomodoro")
          }
        >
          {snapshot.mode === "pomodoro" ? "Countdown" : "Pomodoro"}
        </button>
      </div>
    </main>
  );
}

export default App;
