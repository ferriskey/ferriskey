import { Navigate, Route, Routes } from 'react-router'
import ConsoleNotBuiltYet from '@/next/shell/ciam/not-built-yet'

export default function ConsoleAuthentication() {
  return (
    <Routes>
      <Route index element={<Navigate to='sign-in-methods' replace />} />
      <Route path='sign-in-methods' element={<ConsoleNotBuiltYet section='authentication' page='sign-in-methods' />} />
      <Route path='identity-providers' element={<ConsoleNotBuiltYet section='authentication' page='identity-providers' />} />
      <Route path='password-policy' element={<ConsoleNotBuiltYet section='authentication' page='password-policy' />} />
    </Routes>
  )
}
