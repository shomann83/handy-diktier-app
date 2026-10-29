import React from "react";
import { type StyleBarState, chipLabels } from "./styleBarState";

// Row of equal-width style chips above the pill: "0 Roh · 1 Standard · …".
// The active style is highlighted; once locked (post-processing started) the
// others dim because switching is no longer possible.
const StyleBar: React.FC<{ state: StyleBarState; rawLabel: string }> = ({
  state,
  rawLabel,
}) => (
  <div className={`sbar ${state.locked ? "locked" : ""}`}>
    {chipLabels(state, rawLabel).map((label, i) => (
      <span key={i} className={`schip ${i === state.active ? "on" : ""}`}>
        <kbd>{i}</kbd>
        {label}
      </span>
    ))}
  </div>
);

export default StyleBar;
