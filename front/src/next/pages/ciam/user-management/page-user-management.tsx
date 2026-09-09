import { Navigate, Route, Routes } from 'react-router'
import ConsoleNotBuiltYet from '@/next/shell/ciam/not-built-yet'

export default function ConsoleUserManagement() {
  return (
    <Routes>
      <Route index element={<Navigate to='identities' replace />} />
      <Route path='identities' element={<ConsoleNotBuiltYet section='user-management' page='identities' />} />
      <Route path='organizations' element={<ConsoleNotBuiltYet section='user-management' page='organizations' />} />
      <Route path='roles' element={<ConsoleNotBuiltYet section='user-management' page='roles' />} />
    </Routes>
  )
}
