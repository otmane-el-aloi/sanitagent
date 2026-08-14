import React from "react";
import { TokenStats } from "../types/sanitizer";
import { Zap, ShieldCheck, Cpu } from "lucide-react";

interface StatsBadgeProps {
  stats: TokenStats;
  latencyMs: number;
  isDistilled: boolean;
}

export const StatsBadge: React.FC<StatsBadgeProps> = ({
  stats,
  latencyMs,
  isDistilled,
}) => {
  return (
    <div className="flex items-center gap-2 text-xs font-mono select-none">
      {/* Token Reduction Pill */}
      <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-950/80 border border-emerald-500/30 text-emerald-400 font-semibold shadow-inner">
        <ShieldCheck className="w-3.5 h-3.5 text-emerald-400 animate-pulse" />
        <span>
          {stats.raw_tokens} → {stats.sanitized_tokens} tokens
        </span>
        <span className="bg-emerald-500/20 px-1.5 py-0.5 rounded text-[10px] font-bold text-emerald-300">
          -{stats.reduction_percent}%
        </span>
      </div>

      {/* Latency & Stage Indicator */}
      <div className="flex items-center gap-1.5 px-2 py-1 rounded-full bg-zinc-900/90 border border-zinc-800 text-zinc-400">
        {isDistilled ? (
          <>
            <Cpu className="w-3 h-3 text-purple-400" />
            <span className="text-[11px] text-purple-300 font-medium">Stage 2 (Local LLM)</span>
          </>
        ) : (
          <>
            <Zap className="w-3 h-3 text-amber-400" />
            <span className="text-[11px] text-amber-300 font-medium">Stage 1 (&lt;1ms Rule)</span>
          </>
        )}
        <span className="text-zinc-500 text-[10px]">({latencyMs}ms)</span>
      </div>
    </div>
  );
};
