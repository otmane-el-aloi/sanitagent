import React, { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { SanitizationResult } from "../types/sanitizer";
import { StatsBadge } from "./StatsBadge";
import { DiffView } from "./DiffView";
import {
  ChevronDown,
  ShieldCheck,
  Check,
  Play,
  X,
  Command,
} from "lucide-react";

interface HUDProps {
  result: SanitizationResult | null;
  onHide: () => void;
  onRunTestPrompt: () => void;
}

export const HUD: React.FC<HUDProps> = ({
  result,
  onHide,
  onRunTestPrompt,
}) => {
  const [expanded, setExpanded] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const [progress, setProgress] = useState(100);

  // Reset progress to 100 and collapse drawer whenever a new result arrives
  useEffect(() => {
    setProgress(100);
    setExpanded(false);
  }, [result]);

  // Auto-dismiss countdown (6 seconds), pauses on hover or when expanded
  useEffect(() => {
    if (expanded || isHovered) {
      return;
    }

    const duration = 6000; // 6s auto dismiss
    const intervalTime = 50;
    const step = (intervalTime / duration) * 100;

    const timer = setInterval(() => {
      setProgress((prev) => {
        if (prev <= step) {
          clearInterval(timer);
          onHide();
          return 0;
        }
        return prev - step;
      });
    }, intervalTime);

    return () => clearInterval(timer);
  }, [result, expanded, isHovered, onHide]);

  const handleMouseDown = async (e: React.MouseEvent<HTMLDivElement>) => {
    // Only handle primary left click
    if (e.button !== 0) return;

    // Do not trigger window drag if clicking on interactive controls
    const target = e.target as HTMLElement;
    if (target.closest("button, a, input, select, textarea, [data-no-drag]")) {
      return;
    }

    try {
      const appWindow = getCurrentWindow();
      await appWindow.startDragging();
    } catch (err) {
      console.error("Failed to start dragging window:", err);
    }
  };

  return (
    <div
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      className="w-full flex flex-col items-center select-none font-sans p-2 transition-all duration-300"
    >
      {/* Main Floating Pill Header (macOS Widget Style - Draggable) */}
      <div
        data-tauri-drag-region
        onMouseDown={handleMouseDown}
        className="hud-panel rounded-full px-6 py-2.5 flex flex-row items-center justify-between gap-5 overflow-hidden h-12 w-full max-w-[860px] min-w-[800px] shadow-2xl transition-all duration-300 shrink-0 cursor-grab active:cursor-grabbing"
      >
        {/* Left: App Brand & Status Dot */}
        <div data-tauri-drag-region className="flex items-center gap-2.5 shrink-0">
          <span className="relative flex h-2 w-2 pointer-events-none">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
          </span>

          <div data-tauri-drag-region className="flex items-center gap-1.5 font-semibold text-xs tracking-tight text-white select-none whitespace-nowrap">
            <ShieldCheck className="w-4 h-4 text-emerald-400 pointer-events-none shrink-0" />
            <span data-tauri-drag-region className="whitespace-nowrap">SanitAgent</span>
          </div>
        </div>

        {/* Center: Sleek Token Stats Chip */}
        <div data-tauri-drag-region className="flex items-center justify-center shrink-0 min-w-0">
          {result ? (
            <div data-tauri-drag-region className="min-w-0">
              <StatsBadge
                stats={result.token_stats}
                latencyMs={result.latency_ms}
                isDistilled={result.is_distilled}
              />
            </div>
          ) : (
            <div data-tauri-drag-region className="flex items-center gap-1.5 text-xs text-zinc-400 font-mono bg-zinc-900 border border-zinc-800 px-3 py-1 rounded-full whitespace-nowrap">
              <Command className="w-3 h-3 text-zinc-400 pointer-events-none" />
              <span data-tauri-drag-region className="whitespace-nowrap">Cmd + Shift + S</span>
            </div>
          )}
        </div>

        {/* Right: Copied Badge, Diff Toggle & Controls */}
        <div className="flex items-center gap-2.5 shrink-0">
          {result && (
            <span data-tauri-drag-region className="inline-flex items-center gap-1 text-[11px] font-medium text-emerald-400 bg-emerald-950/80 border border-emerald-800/50 px-2.5 py-0.5 rounded-full whitespace-nowrap">
              <Check className="w-3 h-3 pointer-events-none" />
              Copied
            </span>
          )}

          {result && (
            <button
              onClick={() => setExpanded(!expanded)}
              onMouseDown={(e) => e.stopPropagation()}
              data-tauri-drag-region="false"
              data-no-drag
              className="cursor-pointer flex items-center gap-1 text-xs font-medium px-2.5 py-1 rounded-full bg-zinc-900 hover:bg-zinc-800 text-zinc-300 hover:text-white border border-zinc-800 transition-colors whitespace-nowrap"
              title={expanded ? "Hide Diff" : "Show Unified Diff"}
            >
              <span>Diff</span>
              <ChevronDown
                className={`w-3.5 h-3.5 text-zinc-400 transition-transform duration-200 ${
                  expanded ? "rotate-180" : ""
                }`}
              />
            </button>
          )}

          <button
            onClick={onRunTestPrompt}
            onMouseDown={(e) => e.stopPropagation()}
            data-tauri-drag-region="false"
            data-no-drag
            title="Test with Sample Log + Secret"
            className="cursor-pointer p-1.5 rounded-full hover:bg-zinc-800 text-zinc-400 hover:text-amber-400 transition-colors shrink-0"
          >
            <Play className="w-3.5 h-3.5" />
          </button>

          <button
            onClick={onHide}
            onMouseDown={(e) => e.stopPropagation()}
            data-tauri-drag-region="false"
            data-no-drag
            title="Hide HUD"
            className="cursor-pointer p-1.5 rounded-full hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 transition-colors shrink-0"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Auto Dismiss Progress Line */}
      {!expanded && (
        <div className="w-full max-w-[800px] h-[2px] bg-zinc-900 overflow-hidden rounded-full mt-1">
          <div
            className="h-full bg-gradient-to-r from-emerald-500 to-teal-400 transition-all duration-75"
            style={{ width: `${progress}%` }}
          />
        </div>
      )}

      {/* Expandable Diff Preview Drawer */}
      {expanded && result && (
        <div className="w-full max-w-[860px] mt-2 animate-in fade-in slide-in-from-top-2 duration-200">
          <DiffView
            diffLines={result.diff_lines}
            sanitizedText={result.sanitized_text}
          />
        </div>
      )}
    </div>
  );
};


