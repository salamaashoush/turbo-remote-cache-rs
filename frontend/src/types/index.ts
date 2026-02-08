export interface User {
  id: string;
  email: string;
  name: string;
  role: string;
  email_verified: boolean;
  twofa_method: string;
  twofa_enabled: boolean;
  created_at: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  user: User;
}

export interface RegisterResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  user: User;
  org: Organization | null;
}

export interface TokenResponse {
  access_token: string;
  expires_in: number;
}

export interface Organization {
  id: string;
  name: string;
  slug: string;
  owner_id: string;
  created_at: string;
  updated_at: string;
}

export interface OrgMember {
  org_id: string;
  user_id: string;
  role: string;
  email: string;
  name: string;
  invited_at: string;
  joined_at: string | null;
}

export interface Team {
  id: string;
  org_id: string;
  name: string;
  slug: string;
  created_at: string;
}

export interface TeamMember {
  team_id: string;
  user_id: string;
  role: string;
  email: string;
  name: string;
  added_at: string;
}

export interface ApiToken {
  id: string;
  org_id: string;
  team_id: string | null;
  name: string;
  token_prefix: string;
  scopes: string[];
  expires_at: string | null;
  last_used_at: string | null;
  revoked: boolean;
  created_at: string;
}

export interface CreateTokenResponse {
  id: string;
  name: string;
  token: string;
  token_prefix: string;
  scopes: string[];
  expires_at: string | null;
  created_at: string;
}

export interface AnalyticsOverview {
  period: string;
  total_events: number;
  hits: number;
  misses: number;
  puts: number;
  hit_rate: number;
  total_bytes_saved: number;
  estimated_time_saved_ms: number;
}

export interface TimelinePoint {
  date: string;
  hits: number;
  misses: number;
  puts: number;
}

export interface TeamAnalytics {
  team_id: string | null;
  hits: number;
  misses: number;
  puts: number;
  total_bytes: number;
}

export interface ArtifactEntry {
  artifact_hash: string;
  last_event: string;
  total_events: number;
  total_bytes: number;
  last_seen: string;
}

export interface PlatformStats {
  total_users: number;
  total_orgs: number;
  total_events: number;
  total_bytes: number;
  total_active_tokens: number;
  total_active_sessions: number;
  total_hits: number;
  total_misses: number;
  total_puts: number;
  total_duration_ms: number;
}

export interface OrgAnalytics {
  org_id: string;
  org_name: string;
  org_slug: string;
  hits: number;
  misses: number;
  puts: number;
  total_bytes: number;
}

export interface AdminOrg extends Organization {
  owner_email: string;
  member_count: number;
  team_count: number;
  cache_size_limit_bytes: number | null;
  max_tokens: number | null;
}

export interface AdminUser extends User {
  is_active: boolean;
  org_name: string | null;
  last_active: string | null;
}

export interface AdminUserOrgEntry {
  id: string;
  name: string;
  slug: string;
  role: string;
}

export interface AdminUserDetail {
  id: string;
  email: string;
  name: string;
  role: string;
  is_active: boolean;
  email_verified: boolean;
  twofa_method: string;
  twofa_enabled: boolean;
  created_at: string;
  last_active: string | null;
  orgs: AdminUserOrgEntry[];
  session_count: number;
  token_count: number;
}

export interface AdminOrgOwner {
  id: string;
  email: string;
  name: string;
}

export interface AdminOrgMemberEntry {
  id: string;
  email: string;
  name: string;
  role: string;
}

export interface AdminOrgTeamEntry {
  id: string;
  name: string;
  member_count: number;
}

export interface AdminOrgDetail {
  id: string;
  name: string;
  slug: string;
  owner: AdminOrgOwner;
  members: AdminOrgMemberEntry[];
  teams: AdminOrgTeamEntry[];
  token_count: number;
  cache_size_limit_bytes: number | null;
  max_tokens: number | null;
  total_cache_bytes: number;
  total_cache_events: number;
  created_at: string;
}

export interface UpdateOrgLimitsRequest {
  cache_size_limit_bytes: number | null;
  max_tokens: number | null;
}

// 2FA types
export interface LoginTwofaRequiredResponse {
  twofa_required: true;
  twofa_method: string;
  pending_token: string;
}

export type LoginResponse = AuthResponse | LoginTwofaRequiredResponse;

export interface TotpSetupResponse {
  secret: string;
  qr_code: string;
  otpauth_uri: string;
}

export interface TwofaEnabledResponse {
  recovery_codes: string[];
}

export interface TwofaStatusResponse {
  method: string;
  enabled: boolean;
  recovery_codes_remaining: number;
}

export interface RecoveryCodesResponse {
  recovery_codes: string[];
}
