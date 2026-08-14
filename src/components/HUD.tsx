import React, { useState, useEffect } from "react";
import { SanitizationResult } from "../types/sanitizer";
import { StatsBadge } from "./StatsBadge";
import { DiffView } from "./DiffView";
import {
  ChevronDown,
  ChevronUp,
  Minus,
  Sparkles,
  Play,
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

  // Auto-dismiss countdown (6 seconds), pauses on hover or when expanded
  useEffect(() => {
    if (!result || expanded || isHovered) {
      setProgress(100);
      return;
    }

    const duration = 6000; // 6s auto dismiss
    const intervalTime = 50;
    const step = (intervalTime / duration) * 100;

    const timer = setInterval(() => {
      setProgress((prev) => {
        if (prev <= 0) {
          clearInterval(timer);
          onHide();
          return 0;
        }
        return prev - step;
      });
    }, intervalTime);

    return () => clearInterval(timer);
  }, [result, expanded, isHovered, onHide]);

  return (
    <div
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      className="w-full flex flex-col items-center select-none font-sans p-2 transition-all duration-300"
    >
      {/* Main Floating Pill Header */}
      <div className="hud-glass-panel border border-zinc-800/90 rounded-full px-4 py-2.5 flex items-center justify-between w-full max-w-[540px] shadow-2xl transition-all duration-300">
        {/* Left Status & Brand */}
        <div className="flex items-center gap-2.5">
          <div className="relative flex items-center justify-center">
            <span className="animate-ping absolute inline-flex h-3 w-3 rounded-full bg-emerald-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-emerald-500"></span>
          </div>

          <div className="flex items-center gap-1.5 font-bold text-sm tracking-tight text-zinc-100">
            <Sparkles className="w-4 h-4 text-emerald-400" />
            <span>SanitAgent</span>
          </div>

          <span className="text-[10px] uppercase tracking-wider font-semibold text-emerald-400/90 bg-emerald-950/60 border border-emerald-500/20 px-2 py-0.5 rounded-full">
            Copied
          </span>
        </div>

        {/* Center: Token Stats Badge */}
        {result ? (
          <StatsBadge
            stats={result.token_stats}
            latencyMs={result.latency_ms}
            isDistilled={result.is_distilled}
          />
        ) : (
          <div className="flex items-center gap-1 text-xs text-zinc-400 font-mono">
            <Command className="w-3 h-3 text-zinc-400" />
            <span>Cmd + Shift + S</span>
          </div>
        )}

        {/* Right Controls */}
        <div className="flex items-center gap-1.5">
          {/* Test Trigger Button */}
          <button
            onClick={onRunTestPrompt}
            title="Test with Sample Log + Secret"
            className="p-1.5 rounded-full hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 transition-colors"
          >
            <Play className="w-3.5 h-3.5 text-amber-400" />
          </button>

          {/* Expand Diff Toggle */}
          {result && (
            <button
              onClick={() => setExpanded(!expanded)}
              className="p-1.5 rounded-full hover:bg-zinc-800 text-zinc-300 transition-colors flex items-center gap-1 text-xs font-medium px-2"
              title={expanded ? "Hide Diff" : "Show Unified Diff"}
            >
              <span className="text-[11px] text-zinc-400">
                {expanded ? "Diff" : "Diff"}
              </span>
              {expanded ? (
                <ChevronUp className="w-3.5 h-3.5 text-zinc-400" />
              ) : (
                <ChevronDown className="w-3.5 h-3.5 text-zinc-400" />
              )}
            </button>
          )}

          {/* Minimize / Hide Button */}
          <button
            onClick={onHide}
            title="Minimize HUD (-)"
            className="p-1.5 rounded-full hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200 transition-colors"
          >
            <Minus className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Auto Dismiss Progress Line */}
      {result && !expanded && (
        <div className="w-[500px] h-[2px] bg-zinc-900 overflow-hidden rounded-full mt-1">
          <div
            className="h-full bg-gradient-to-r from-emerald-500 to-teal-400 transition-all duration-75"
            style={{ width: `${progress}%` }}
          />
        </div>
      )}

      {/* Expandable Diff Preview Drawer */}
      {expanded && result && (
        <div className="w-full max-w-[540px] mt-2 animate-in fade-in slide-in-from-top-2 duration-200">
          <DiffView
            diffLines={result.diff_lines}
            sanitizedText={result.sanitized_text}
          />
        </div>
      )}
    </div>
  );
};
