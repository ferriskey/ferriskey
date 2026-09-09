import { Navigate, Route, Routes } from 'react-router'
import ConsoleNotBuiltYet from '@/next/shell/ciam/not-built-yet'

export default function ConsoleActivity() {
  return (
    <Routes>
      <Route index element={<Navigate to='live' replace />} />
      <Route path='live' element={<ConsoleNotBuiltYet section='activity' page='live' />} />
      <Route path='logs' element={<ConsoleNotBuiltYet section='activity' page='logs' />} />
      <Route path='sessions' element={<ConsoleNotBuiltYet section='activity' page='sessions' />} />
      <Route path='messages' element={<ConsoleNotBuiltYet section='activity' page='messages' />} />
    </Routes>
  )
}
