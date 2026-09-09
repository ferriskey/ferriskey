import { Navigate, Route, Routes } from 'react-router'
import PageIdentityProviders from '@/pages/iam/identity-providers/page-identity-providers'
import PageSignInMethodsFeature from './feature/page-sign-in-methods-feature'
import PagePasswordPolicyFeature from './feature/page-password-policy-feature'

export default function ConsoleAuthentication() {
  return (
    <Routes>
      <Route index element={<Navigate to='sign-in-methods' replace />} />
      <Route path='sign-in-methods' element={<PageSignInMethodsFeature />} />
      <Route path='identity-providers/*' element={<PageIdentityProviders />} />
      <Route path='password-policy' element={<PagePasswordPolicyFeature />} />
    </Routes>
  )
}
