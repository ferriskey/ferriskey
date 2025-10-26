import { Route, Routes } from 'react-router-dom'
import PageLoginFeature from './feature/page-login-feature'
import PageCallbackFeature from './feature/page-callback-feature'
import PageRequiredActionFeature from './feature/page-required-action-feature'
import PageOtpChallengeFeature from './feature/page-otp-challenge-feature'
import PageRegisterFeature from './feature/page-register-feature'

export default function PageAuthentication() {
  return (
    <Routes>
      <Route path='/login' element={<PageLoginFeature />} />
      <Route path='/register' element={<PageRegisterFeature />} />
      <Route path='/callback' element={<PageCallbackFeature />} />
      <Route path='/required-action' element={<PageRequiredActionFeature />} />
      <Route path='/otp' element={<PageOtpChallengeFeature />} />
    </Routes>
  )
}
