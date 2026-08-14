import React from "react";
import { TokenStats } from "../types/sanitizer";
import { Zap, Sparkles, Shield } from "lucide-react";

interface StatsBadgeProps {
  stats: TokenStats;
  latencyMs: number;
  isDistilled?: boolean;
}

export const StatsBadge: React.FC<StatsBadgeProps> = ({
  stats,
  latencyMs,
  isDistilled,
}) => {
  return (
    <div className="inline-flex items-center gap-2.5 px-3.5 py-1 rounded-full bg-zinc-900 border border-zinc-800 text-xs font-mono text-zinc-300 whitespace-nowrap shadow-sm">
      {/* Pipeline Stage Badge */}
      {isDistilled ? (
        <span
          title="Stage 2: Ollama AI Local LLM Distillation"
          className="inline-flex items-center gap-1 text-[10px] font-semibold text-purple-300 bg-purple-950/80 border border-purple-800/60 px-2 py-0.5 rounded-md whitespace-nowrap"
        >
          <Sparkles className="w-2.5 h-2.5 text-purple-400 shrink-0" />
          AI Distilled
        </span>
      ) : (
        <span
          title="Stage 1: Deterministic Rule Cleaner (<1ms)"
          className="inline-flex items-center gap-1 text-[10px] font-semibold text-teal-300 bg-teal-950/80 border border-teal-800/60 px-2 py-0.5 rounded-md whitespace-nowrap"
        >
          <Shield className="w-2.5 h-2.5 text-teal-400 shrink-0" />
          Stage 1 Rule
        </span>
      )}

      <span className="text-zinc-200 font-medium whitespace-nowrap">
        {stats.raw_tokens} → {stats.sanitized_tokens} tokens
      </span>
      <span className="text-emerald-400 font-semibold bg-emerald-950 border border-emerald-800/50 px-1.5 py-0.5 rounded-md text-[11px] whitespace-nowrap">
        -{stats.reduction_percent}%
      </span>
      {latencyMs > 0 && (
        <span className="flex items-center gap-1 text-[10px] text-zinc-500 border-l border-zinc-800 pl-2 whitespace-nowrap">
          <Zap className="w-2.5 h-2.5 text-zinc-400 shrink-0" />
          {latencyMs}ms
        </span>
      )}
    </div>
  );
};


