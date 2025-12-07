import { ReactNode } from 'react'
import { Outlet } from 'react-router'

interface PageContainerProps {
  children?: ReactNode
}

export default function PageContainer({ children }: PageContainerProps) {
  return (
    <div className='flex flex-col gap-6 p-8'>
      {children}
      <Outlet />
    </div>
  )
}
