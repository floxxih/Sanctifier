"use client";

import { useCallback, useMemo, useRef } from "react";
import { FixedSizeList, type ListChildComponentProps } from "react-window";
import type { Finding, Severity } from "../types";
import { CodeSnippet } from "./CodeSnippet";
import { Sparkles } from "lucide-react";
import { AiFixPanel } from "./AiFixPanel";

interface FindingsListProps {
  findings: Finding[];
  severityFilter: Severity | "all";
}

/** Height reserved for each finding row in the virtual list (px). */
const ITEM_HEIGHT = 140;
/** Maximum height of the scrollable list before it starts scrolling. */
const MAX_LIST_HEIGHT = 600;
/** Below this threshold we skip virtualisation and render items directly. */
const VIRTUALISE_THRESHOLD = 50;

const severityColors: Record<Severity, string> = {
  critical: "bg-red-500/10 border-red-500/50 text-red-700 dark:text-red-400 theme-high-contrast:bg-black theme-high-contrast:border-white theme-high-contrast:text-white",
  high: "bg-orange-500/10 border-orange-500/50 text-orange-700 dark:text-orange-400 theme-high-contrast:bg-black theme-high-contrast:border-white theme-high-contrast:text-white",
  medium: "bg-amber-500/10 border-amber-500/50 text-amber-700 dark:text-amber-400 theme-high-contrast:bg-black theme-high-contrast:border-white theme-high-contrast:text-white",
  low: "bg-zinc-500/10 border-zinc-500/50 text-zinc-700 dark:text-zinc-400 theme-high-contrast:bg-black theme-high-contrast:border-yellow-300 theme-high-contrast:text-yellow-300",
};

const severityLabels: Record<Severity, string> = {
  critical: "Critical severity",
  high: "High severity",
  medium: "Medium severity",
  low: "Low severity",
};

function FindingCard({ finding }: { finding: Finding }) {
  return (
    <div className={`rounded-lg border p-4 ${severityColors[finding.severity]}`}>
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <span className="text-xs font-semibold uppercase tracking-wide opacity-80">
            {finding.category}
          </span>
          <h3 className="mt-1 font-medium">{finding.title}</h3>
          <p className="mt-1 text-sm opacity-90">{finding.location}</p>
          {finding.suggestion && (
            <p className="mt-2 text-sm italic">💡 {finding.suggestion}</p>
          )}
        </div>
        <div className="shrink-0 flex items-center gap-2">
          <span
            className={`rounded px-2 py-1 text-xs font-medium border ${severityColors[finding.severity]}`}
            aria-label={severityLabels[finding.severity]}
          >
            {finding.severity}
          </span>
          <span className="font-mono text-xs rounded border border-zinc-300/70 dark:border-zinc-600 px-2 py-1 text-zinc-700 dark:text-zinc-300 theme-high-contrast:border-white theme-high-contrast:text-white">
            {finding.code}
          </span>
        </div>
      </div>
      {finding.snippet && (
        <div className="mt-3">
          <CodeSnippet code={finding.snippet} highlightLine={finding.line} />
        </div>
      )}
    </div>
  );
}

export function FindingsList({ findings, severityFilter }: FindingsListProps) {
  const listRef = useRef<FixedSizeList>(null);
  const filtered = useMemo(() => {
    return severityFilter === "all"
      ? findings
      : findings.filter((f) => f.severity === severityFilter);
  }, [findings, severityFilter]);

  // Scroll back to top whenever the filter changes.
  const prevFilterRef = useRef(severityFilter);
  if (prevFilterRef.current !== severityFilter) {
    prevFilterRef.current = severityFilter;
    listRef.current?.scrollToItem(0);
  }

  const Row = useCallback(
    ({ index, style }: ListChildComponentProps) => (
      <div style={{ ...style, paddingBottom: 16 }}>
        <FindingCard finding={filtered[index]} />
      </div>
    ),
    [filtered],
  );

  if (filtered.length === 0) {
    return (
      <p className="text-zinc-500 dark:text-zinc-400 theme-high-contrast:text-white py-8 text-center">
        No findings match the selected filter.
      </p>
    );
  }

  // For small lists render items directly — no virtualisation overhead.
  if (filtered.length < VIRTUALISE_THRESHOLD) {
    return (
      <div className="space-y-4">
        {filtered.map((f) => (
          <FindingCard key={f.id} finding={f} />
        ))}
      </div>
    );
  }

  // For large lists (1000+) use a fixed-size virtual window.
  const listHeight = Math.min(filtered.length * ITEM_HEIGHT, MAX_LIST_HEIGHT);

  return (
    <FixedSizeList
      height={listHeight}
      itemCount={filtered.length}
      itemSize={ITEM_HEIGHT}
      width="100%"
      ref={listRef}
    >
      {Row}
    </FixedSizeList>
  );
}
