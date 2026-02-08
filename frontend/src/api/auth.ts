import { apiFetch, setTokens } from './client';
import type {
  AuthResponse,
  LoginResponse,
  LoginTwofaRequiredResponse,
  RecoveryCodesResponse,
  RegisterResponse,
  TotpSetupResponse,
  TwofaEnabledResponse,
  TwofaStatusResponse,
  User,
} from '../types';

export async function register(
  email: string,
  password: string,
  name: string,
): Promise<RegisterResponse> {
  const res = await apiFetch<RegisterResponse>('/auth/register', {
    method: 'POST',
    body: JSON.stringify({ email, password, name }),
  });
  setTokens(res.access_token, res.refresh_token);
  return res;
}

function isTwofaRequired(res: LoginResponse): res is LoginTwofaRequiredResponse {
  return 'twofa_required' in res && res.twofa_required === true;
}

export async function login(email: string, password: string): Promise<LoginResponse> {
  const res = await apiFetch<LoginResponse>('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ email, password }),
  });
  if (!isTwofaRequired(res)) {
    setTokens(res.access_token, res.refresh_token);
  }
  return res;
}

export async function verifyTwofaLogin(
  pending_token: string,
  code: string,
): Promise<AuthResponse> {
  const res = await apiFetch<AuthResponse>('/auth/2fa/verify-login', {
    method: 'POST',
    body: JSON.stringify({ pending_token, code }),
  });
  setTokens(res.access_token, res.refresh_token);
  return res;
}

export async function getMe(): Promise<User> {
  return apiFetch<User>('/auth/me');
}

export async function updateProfile(data: { name?: string; password?: string }): Promise<User> {
  return apiFetch<User>('/auth/me', {
    method: 'PATCH',
    body: JSON.stringify(data),
  });
}

export async function forgotPassword(email: string): Promise<{ message: string }> {
  return apiFetch<{ message: string }>('/auth/forgot-password', {
    method: 'POST',
    body: JSON.stringify({ email }),
  });
}

export async function resetPassword(
  token: string,
  new_password: string,
): Promise<{ message: string }> {
  return apiFetch<{ message: string }>('/auth/reset-password', {
    method: 'POST',
    body: JSON.stringify({ token, new_password }),
  });
}

export async function verifyEmail(token: string): Promise<{ message: string }> {
  return apiFetch<{ message: string }>('/auth/verify-email', {
    method: 'POST',
    body: JSON.stringify({ token }),
  });
}

// 2FA management

export async function getTwofaStatus(): Promise<TwofaStatusResponse> {
  return apiFetch<TwofaStatusResponse>('/auth/2fa/status');
}

export async function setupTotp(): Promise<TotpSetupResponse> {
  return apiFetch<TotpSetupResponse>('/auth/2fa/totp/setup', {
    method: 'POST',
  });
}

export async function confirmTotp(code: string): Promise<TwofaEnabledResponse> {
  return apiFetch<TwofaEnabledResponse>('/auth/2fa/totp/confirm', {
    method: 'POST',
    body: JSON.stringify({ code }),
  });
}

export async function enableEmailTwofa(): Promise<TwofaEnabledResponse> {
  return apiFetch<TwofaEnabledResponse>('/auth/2fa/email/enable', {
    method: 'POST',
  });
}

export async function disableTwofa(password: string): Promise<{ message: string }> {
  return apiFetch<{ message: string }>('/auth/2fa/disable', {
    method: 'POST',
    body: JSON.stringify({ password }),
  });
}

export async function regenerateRecoveryCodes(
  password: string,
): Promise<RecoveryCodesResponse> {
  return apiFetch<RecoveryCodesResponse>('/auth/2fa/recovery/regenerate', {
    method: 'POST',
    body: JSON.stringify({ password }),
  });
}
