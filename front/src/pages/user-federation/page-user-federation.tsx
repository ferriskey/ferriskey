import { Navigate, Route, Routes } from 'react-router'
import PageOverviewFeature from './feature/page-overview-feature'
import PageCreateLdapFeature from './feature/page-create-ldap-feature'
import PageCreateKerberosFeature from './feature/page-create-kerberos-feature'
import PageDetailLdapFeature from './feature/page-detail-ldap-feature'

export default function PageUserFederation() {
  return (
    <Routes>
      <Route index element={<PageOverviewFeature />} />
      <Route path='/overview' element={<Navigate to='..' replace />} />
      <Route path='/ldap/create' element={<PageCreateLdapFeature />} />
      <Route path='/ldap/:id' element={<PageDetailLdapFeature />} />
      <Route path='/kerberos/create' element={<PageCreateKerberosFeature />} />
      <Route path='/create' element={<Navigate to='../ldap/create' replace />} />
    </Routes>
  )
}
