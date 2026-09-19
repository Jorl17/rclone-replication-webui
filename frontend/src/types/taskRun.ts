/** Statistiques extraites des logs JSON rclone (dernière ligne "stats") */
export interface RcloneStats {
  bytes: number;
  checks: number;
  deletedDirs: number;
  deletes: number;
  elapsedTime: number;
  errors: number;
  fatalError: boolean;
  renames: number;
  retryError: boolean;
  speed: number;
  totalBytes: number;
  totalChecks: number;
  totalTransfers: number;
  transferTime: number;
  transfers: number;
}

export interface TaskRunSummary {
  id: string;
  task_id: string;
  triggered_by: 'scheduler' | 'manual' | 'restore';
  status: 'running' | 'success' | 'failure';
  started_at: string;
  finished_at: string | null;
  duration_ms: number | null;
  exit_code: number | null;
  stats: RcloneStats | null;
}

export interface TaskRun extends TaskRunSummary {
  log_bytes?: number;
}

export interface RunLogLine {
  offset: number;
  text: string;
}

export interface RunLogPage {
  lines: RunLogLine[];
  total_bytes: number;
  at_start: boolean;
  at_end: boolean;
  prev_offset: number | null;
  next_offset: number | null;
}

export type RunLogQuery =
  | { from: 'start' | 'end' }
  | { after: number }
  | { before: number };
