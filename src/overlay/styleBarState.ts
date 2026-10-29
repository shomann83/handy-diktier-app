// State of the per-dictation style bar, mirrored from the Rust `StyleBarEvent`
// (emitted as "style-bar-event"). Slot 0 is raw (no LLM), slots 1-9 are the
// post-processing prompts in list order.
export type StyleBarState = {
  visible: boolean;
  labels: string[];
  active: number;
  locked: boolean;
};

export const HIDDEN: StyleBarState = {
  visible: false,
  labels: [],
  active: 0,
  locked: false,
};

/** Chip texts, with the raw slot replaced by its localized label. */
export function chipLabels(state: StyleBarState, rawLabel: string): string[] {
  return state.labels.map((label, i) => (i === 0 ? rawLabel : label));
}

/** Label for the post-processing phase: names the style while the bar shows. */
export function processingLabel(
  state: StyleBarState,
  fallback: string,
  withStyle: (style: string) => string,
): string {
  const style = state.visible ? state.labels[state.active] : undefined;
  return style ? withStyle(style) : fallback;
}
