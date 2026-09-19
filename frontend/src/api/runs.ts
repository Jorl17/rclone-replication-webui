import client from './client';
import { RunLogPage, RunLogQuery, TaskRun, TaskRunSummary } from '../types/taskRun';

export const getTaskRuns = (taskId: string) => client.get<TaskRunSummary[]>(`/tasks/${taskId}/runs`).then(r => r.data);
export const getRun = (runId: string) => client.get<TaskRun>(`/runs/${runId}`).then(r => r.data);

export const getRunLogs = (runId: string, query: RunLogQuery) => {
  const params =
    'from' in query
      ? { from: query.from }
      : 'after' in query
        ? { after: query.after }
        : { before: query.before };
  return client.get<RunLogPage>(`/runs/${runId}/logs`, { params }).then(r => r.data);
};
