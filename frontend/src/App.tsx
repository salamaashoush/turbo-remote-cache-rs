import { Routes, Route, Navigate } from 'react-router-dom';
import { useAuth } from './hooks/useAuth';
import { Layout } from './components/Layout';
import { ProtectedRoute } from './components/ProtectedRoute';
import { Login } from './pages/Login';
import { Register } from './pages/Register';
import { ForgotPassword } from './pages/ForgotPassword';
import { ResetPassword } from './pages/ResetPassword';
import { VerifyEmail } from './pages/VerifyEmail';
import { Dashboard } from './pages/Dashboard';
import { OrgOverview } from './pages/OrgOverview';
import { OrgSettings } from './pages/OrgSettings';
import { TeamManagement } from './pages/TeamManagement';
import { TokenManagement } from './pages/TokenManagement';
import { CacheAnalytics } from './pages/CacheAnalytics';
import { ArtifactBrowser } from './pages/ArtifactBrowser';
import { UserProfile } from './pages/UserProfile';
import { AdminOrgs } from './pages/admin/AdminOrgs';
import { AdminUsers } from './pages/admin/AdminUsers';
import { AdminStats } from './pages/admin/AdminStats';
import { AdminUserDetail } from './pages/admin/AdminUserDetail';
import { AdminOrgDetail } from './pages/admin/AdminOrgDetail';

function AdminRoute({ user, children }: { user: { role: string } | null; children: React.ReactNode }) {
  if (!user || user.role !== 'super_admin') {
    return <Navigate to="/" replace />;
  }
  return <>{children}</>;
}

export function App() {
  const { user, loading, logout, refetch } = useAuth();

  return (
    <Routes>
      <Route path="/login" element={user ? <Navigate to="/" replace /> : <Login onLogin={refetch} />} />
      <Route path="/register" element={user ? <Navigate to="/" replace /> : <Register onLogin={refetch} />} />
      <Route path="/forgot-password" element={<ForgotPassword />} />
      <Route path="/reset-password" element={<ResetPassword />} />
      <Route path="/verify-email" element={<VerifyEmail />} />

      <Route
        path="/*"
        element={
          <ProtectedRoute user={user} loading={loading}>
            <Layout user={user!} onLogout={logout}>
              <Routes>
                <Route path="/" element={<Dashboard user={user!} />} />
                <Route path="/profile" element={<UserProfile user={user!} onUpdate={refetch} />} />

                <Route path="/orgs/:orgId" element={<OrgOverview />} />
                <Route path="/orgs/:orgId/settings" element={<OrgSettings />} />
                <Route path="/orgs/:orgId/teams" element={<TeamManagement />} />
                <Route path="/orgs/:orgId/tokens" element={<TokenManagement />} />
                <Route path="/orgs/:orgId/analytics" element={<CacheAnalytics />} />
                <Route path="/orgs/:orgId/cache" element={<ArtifactBrowser />} />

                <Route path="/admin/orgs" element={<AdminRoute user={user}><AdminOrgs /></AdminRoute>} />
                <Route path="/admin/orgs/:orgId" element={<AdminRoute user={user}><AdminOrgDetail /></AdminRoute>} />
                <Route path="/admin/users" element={<AdminRoute user={user}><AdminUsers /></AdminRoute>} />
                <Route path="/admin/users/:userId" element={<AdminRoute user={user}><AdminUserDetail /></AdminRoute>} />
                <Route path="/admin/stats" element={<AdminRoute user={user}><AdminStats /></AdminRoute>} />

                <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            </Layout>
          </ProtectedRoute>
        }
      />
    </Routes>
  );
}
