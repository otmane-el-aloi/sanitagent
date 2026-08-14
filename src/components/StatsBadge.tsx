import React from "react";
import { TokenStats } from "../types/sanitizer";
import { Zap } from "lucide-react";

interface StatsBadgeProps {
  stats: TokenStats;
  latencyMs: number;
  isDistilled?: boolean;
}

export const StatsBadge: React.FC<StatsBadgeProps> = ({
  stats,
  latencyMs,
}) => {
  return (
    <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-zinc-900 border border-zinc-800 text-xs font-mono text-zinc-300 whitespace-nowrap shadow-sm">
      <span className="text-zinc-200 font-medium">
        {stats.raw_tokens} → {stats.sanitized_tokens} tokens
      </span>
      <span className="text-emerald-400 font-semibold bg-emerald-950 border border-emerald-800/50 px-1.5 py-0.5 rounded-md text-[11px]">
        -{stats.reduction_percent}%
      </span>
      {latencyMs > 0 && (
        <span className="flex items-center gap-1 text-[10px] text-zinc-500 border-l border-zinc-800 pl-2">
          <Zap className="w-2.5 h-2.5 text-zinc-400" />
          {latencyMs}ms
        </span>
      )}
    </div>
  );
};

