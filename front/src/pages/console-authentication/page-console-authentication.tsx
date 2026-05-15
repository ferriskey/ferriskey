import ConsoleComingSoon from '@/components/console-coming-soon'
import { Navigate, Route, Routes } from 'react-router'

export default function PageConsoleAuthentication() {
  return (
    <Routes>
      <Route index element={<Navigate to='sign-in-methods' replace />} />
      <Route
        path='sign-in-methods'
        element={
          <ConsoleComingSoon
            title='Sign-in methods'
            description='Configure passkey, magic link, password and MFA options for this realm.'
          />
        }
      />
      <Route
        path='identity-providers'
        element={
          <ConsoleComingSoon
            title='Identity providers'
            description='Let customers sign in with social accounts and enterprise SSO.'
          />
        }
      />
      <Route
        path='user-federation'
        element={
          <ConsoleComingSoon
            title='User federation'
            description='Sync identities from external directories like LDAP or SCIM.'
          />
        }
      />
      <Route
        path='password-policy'
        element={
          <ConsoleComingSoon
            title='Password policy'
            description='Set strength requirements and rotation rules for passwords.'
          />
        }
      />
    </Routes>
  )
}
