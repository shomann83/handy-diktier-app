import assert from "node:assert/strict";
import { HIDDEN, chipLabels, processingLabel } from "./styleBarState";

// raw chip is localized
assert.deepEqual(
  chipLabels(
    { visible: true, labels: ["Roh", "Standard"], active: 1, locked: false },
    "Raw",
  ),
  ["Raw", "Standard"],
);

// processing label names the active style
assert.equal(
  processingLabel(
    {
      visible: true,
      labels: ["Roh", "Standard", "E-Mail"],
      active: 2,
      locked: true,
    },
    "Verarbeite...",
    (s) => `KI bereitet auf (${s})…`,
  ),
  "KI bereitet auf (E-Mail)…",
);

// processing label falls back when the bar is hidden
assert.equal(
  processingLabel(HIDDEN, "Verarbeite...", (s) => s),
  "Verarbeite...",
);

console.log("styleBarState: all assertions passed");
