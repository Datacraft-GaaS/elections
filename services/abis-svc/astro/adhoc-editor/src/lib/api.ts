/**
 * Copyright (c) 2026 Nyimbi Odero
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * Thin fetch wrapper for the adhoc reports API.
 * Uses a SPIRE workload token from the /whoami same-origin endpoint.
 */

const BASE = import.meta.env.PUBLIC_ADHOC_API_URL ?? '/v1/adhoc';

async function getToken(): Promise<string | null> {
  try {
    const res = await fetch('/whoami', { credentials: 'include' });
    if (!res.ok) return null;
    const data = await res.json();
    return data?.token ?? null;
  } catch {
    return null;
  }
}

async function apiFetch(path: string, opts: RequestInit = {}): Promise<Response> {
  const token = await getToken();
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(opts.headers as Record<string, string> ?? {}),
  };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  return fetch(`${BASE}${path}`, { ...opts, headers });
}

export interface AdhocReport {
  adhoc_id: string;
  slug: string;
  title: string;
  description?: string;
  visibility: 'private' | 'shared' | 'team' | 'org';
  data_class: 'restricted' | 'sensitive' | 'public';
  updated_at: string;
  archived_at?: string;
}

export interface AdhocReportVersion {
  version_id: string;
  adhoc_id: string;
  version_number: number;
  source_type: 'sql' | 'query_builder' | 'markdown';
  source: string;
  allowed_tables: string[];
  safety_score: number;
  lint_report: LintReport;
  created_at: string;
}

export interface LintReport {
  errors: Diagnostic[];
  warnings: Diagnostic[];
  safety_score: number;
  modified_sql?: string;
}

export interface Diagnostic {
  code: string;
  message: string;
  severity: 'error' | 'warning';
}

export interface PreviewResult {
  preview_id: string;
  rows?: unknown[];
  preview_format: string;
}

export const api = {
  listReports: async (): Promise<AdhocReport[]> => {
    const res = await apiFetch('/reports');
    if (!res.ok) return [];
    const data = await res.json();
    return data.reports ?? [];
  },

  createReport: async (body: {
    slug: string;
    title: string;
    description?: string;
    visibility: string;
    data_class: string;
  }): Promise<{ adhoc_id?: string }> => {
    const res = await apiFetch('/reports', { method: 'POST', body: JSON.stringify(body) });
    return res.json();
  },

  createVersion: async (
    adhocId: string,
    body: {
      source: string;
      source_type: string;
      allowed_tables: string[];
      commit_message?: string;
    }
  ): Promise<AdhocReportVersion> => {
    const res = await apiFetch(`/reports/${adhocId}/versions`, {
      method: 'POST',
      body: JSON.stringify(body),
    });
    return res.json();
  },

  runPreview: async (
    versionId: string,
    body: { format?: string; sample_limit?: number }
  ): Promise<PreviewResult> => {
    const res = await apiFetch(`/versions/${versionId}/preview`, {
      method: 'POST',
      body: JSON.stringify(body),
    });
    return res.json();
  },

  publishVersion: async (
    versionId: string,
    body: { formats: string[]; schedule?: string }
  ): Promise<{ definition_id?: string }> => {
    const res = await apiFetch(`/versions/${versionId}/publish`, {
      method: 'POST',
      body: JSON.stringify(body),
    });
    return res.json();
  },
};
