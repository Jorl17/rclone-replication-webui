import { useQuery, useMutation, useQueryClient, keepPreviousData } from '@tanstack/react-query';
import * as api from '../api/notifications';
import { ChannelTemplates, CreateChannelPayload } from '../types/notification';
import type { SupportedLanguage } from '../i18n';

export function useNotifications() {
  return useQuery({ queryKey: ['notifications'], queryFn: api.getChannels });
}

export function useCreateChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (payload: CreateChannelPayload) => api.createChannel(payload),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['notifications'] }),
  });
}

export function useUpdateChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, payload }: { id: string; payload: CreateChannelPayload }) => api.updateChannel(id, payload),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['notifications'] }),
  });
}

export function useDeleteChannel() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteChannel(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['notifications'] }),
  });
}

export function useTestChannel() {
  return useMutation({ mutationFn: (id: string) => api.testChannel(id) });
}

export function useNotificationPreview(
  language: SupportedLanguage,
  templates: ChannelTemplates,
  enabled = true,
) {
  return useQuery({
    queryKey: ['notification-preview', language, templates],
    queryFn: () => api.previewNotification({ language, templates }),
    enabled,
    placeholderData: keepPreviousData,
  });
}
