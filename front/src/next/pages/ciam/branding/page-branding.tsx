import { Navigate, Route, Routes } from 'react-router'
import ConsoleNotBuiltYet from '@/next/shell/ciam/not-built-yet'

export default function ConsoleBranding() {
  return (
    <Routes>
      <Route index element={<Navigate to='email-templates' replace />} />
      <Route path='email-templates' element={<ConsoleNotBuiltYet section='branding' page='email-templates' />} />
      <Route path='themes' element={<ConsoleNotBuiltYet section='branding' page='themes' />} />
    </Routes>
  )
}
