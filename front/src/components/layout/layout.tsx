import { Outlet, useParams } from 'react-router'
import { AppSidebar } from '../app-sidebar'
import { SidebarInset, SidebarProvider } from '../ui/sidebar'
import { useGetUserRealmsQuery } from '@/api/realm.api'
import { RouterParams } from '@/routes/router'
import useRealmStore from '@/store/realm.store'
import { useEffect } from 'react'
import GithubStarModal from '../github-star-modal'
import { TopBar } from '../top-bar'

export default function Layout() {
  const { realm_name } = useParams<RouterParams>()
  const { setUserRealms } = useRealmStore()
  const { data: userRealmsResponse } = useGetUserRealmsQuery({ realm: realm_name ?? 'master' })

  useEffect(() => {
    if (userRealmsResponse) {
      setUserRealms(userRealmsResponse.data)
    }
  }, [userRealmsResponse, setUserRealms])

  /* Le style FerrisKey ne s'applique qu'à la console IAM : le drapeau est posé
     ici, jamais par `product-layout` (CIAM), qui garde donc son rendu actuel.
     Sur <html> plutôt que sur un conteneur de page pour que les portails —
     popover, tooltip, dialog, rendus hors du shell — en héritent aussi. */
  useEffect(() => {
    document.documentElement.dataset.style = 'ferriskey'
    return () => {
      delete document.documentElement.dataset.style
    }
  }, [])

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <TopBar />
        <Outlet />
      </SidebarInset>

      <GithubStarModal />
    </SidebarProvider>
  )
}
