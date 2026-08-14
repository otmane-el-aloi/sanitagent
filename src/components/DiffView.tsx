import React, { useState } from "react";
import { DiffLine } from "../types/sanitizer";
import { Check, Copy, FileText, Search } from "lucide-react";

interface DiffViewProps {
  diffLines: DiffLine[];
  sanitizedText: string;
}

export const DiffView: React.FC<DiffViewProps> = ({ diffLines, sanitizedText }) => {
  const [copied, setCopied] = useState(false);
  const [searchFilter, setSearchFilter] = useState("");

  const handleCopy = () => {
    navigator.clipboard.writeText(sanitizedText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const filteredLines = searchFilter
    ? diffLines.filter((l) =>
        l.content.toLowerCase().includes(searchFilter.toLowerCase())
      )
    : diffLines;

  return (
    <div className="flex flex-col h-full bg-zinc-950 border border-zinc-800 rounded-xl overflow-hidden shadow-2xl">
      {/* Diff Toolbar */}
      <div className="flex items-center justify-between px-3 py-2 bg-zinc-900 border-b border-zinc-800">
        <div className="flex items-center gap-2 text-xs font-medium text-zinc-300">
          <FileText className="w-3.5 h-3.5 text-zinc-400" />
          <span>Unified Diff Preview</span>
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400 font-mono">
            {diffLines.length} lines
          </span>
        </div>

        <div className="flex items-center gap-2">
          {/* Search Filter */}
          <div className="relative flex items-center">
            <Search className="w-3 h-3 absolute left-2 text-zinc-500" />
            <input
              type="text"
              placeholder="Filter diff..."
              value={searchFilter}
              onChange={(e) => setSearchFilter(e.target.value)}
              className="w-28 focus:w-36 transition-all duration-200 bg-zinc-950 border border-zinc-800 rounded-md pl-6 pr-2 py-0.5 text-[11px] text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-zinc-600"
            />
          </div>

          {/* Copy Clean Output Button */}
          <button
            onClick={handleCopy}
            className="flex items-center gap-1 text-[11px] font-medium px-2 py-1 rounded-md bg-zinc-800 hover:bg-zinc-700 text-zinc-200 transition-colors"
          >
            {copied ? (
              <>
                <Check className="w-3 h-3 text-emerald-400" />
                <span className="text-emerald-400">Copied!</span>
              </>
            ) : (
              <>
                <Copy className="w-3 h-3 text-zinc-400" />
                <span>Copy Output</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Diff Content Box */}
      <div className="flex-1 overflow-y-auto max-h-[300px] font-mono text-[11px] leading-5 select-text p-2 space-y-0.5">
        {filteredLines.length === 0 ? (
          <div className="text-center py-6 text-zinc-500 italic text-xs">
            No matching diff lines found.
          </div>
        ) : (
          filteredLines.map((line, idx) => {
            let bgClass = "hover:bg-zinc-900 text-zinc-300";
            let prefixChar = " ";
            let prefixColor = "text-zinc-600";

            if (line.change_type === "delete") {
              bgClass = "bg-red-950 text-red-300 border-l-2 border-red-500 font-semibold";
              prefixChar = "-";
              prefixColor = "text-red-400 font-bold";
            } else if (line.change_type === "add") {
              bgClass = "bg-emerald-950 text-emerald-300 border-l-2 border-emerald-500 font-semibold";
              prefixChar = "+";
              prefixColor = "text-emerald-400 font-bold";
            }

            return (
              <div
                key={idx}
                className={`flex items-start px-2 py-0.5 rounded-sm transition-colors ${bgClass}`}
              >
                {/* Line numbers */}
                <div className="flex select-none gap-2 min-w-[48px] text-[10px] text-zinc-500 font-mono pt-0.5 shrink-0">
                  <span className="w-5 text-right inline-block">
                    {line.old_line_no ?? ""}
                  </span>
                  <span className="w-5 text-right inline-block">
                    {line.new_line_no ?? ""}
                  </span>
                </div>

                {/* Diff Prefix Sign */}
                <span className={`w-4 text-center select-none shrink-0 ${prefixColor}`}>
                  {prefixChar}
                </span>

                {/* Line content */}
                <span className="whitespace-pre-wrap break-all flex-1 font-mono text-zinc-200">
                  {line.content || " "}
                </span>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
