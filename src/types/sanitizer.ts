export interface TokenStats {
  raw_tokens: number;
  sanitized_tokens: number;
  saved_tokens: number;
  reduction_percent: number;
}

export interface DiffLine {
  change_type: "add" | "delete" | "equal";
  old_line_no: number | null;
  new_line_no: number | null;
  content: string;
}

export interface SanitizationResult {
  raw_text: string;
  sanitized_text: string;
  stage1_text: string;
  is_distilled: boolean;
  latency_ms: number;
  token_stats: TokenStats;
  diff_lines: DiffLine[];
}
