import { Client, User, Role } from './api.types'

/**
 * Home page data interface using OpenAPI generated types
 */
export interface PageHomeData {
  clients: Client[]
  users: User[]
  roles: Role[]
  isLoading: boolean
}

/**
 * Calculated metrics for the home dashboard
 */
export interface HomeMetrics {
  totalClients: number
  totalUsers: number
  totalRoles: number
  activeClients: number
  serviceAccountClients: number
  clientsActivePercentage: number
}

/**
 * Chart data structure for pie chart
 */
export interface ChartDataItem {
  name: string
  value: number
  fill: string
}

/**
 * Chart configuration for ShadCN charts
 */
export interface HomeChartConfig {
  [key: string]: {
    label: string
    color: string
  }
  clients: {
    label: string
    color: string
  }
  users: {
    label: string
    color: string
  }
  roles: {
    label: string
    color: string
  }
}

/**
 * Quick access navigation item
 */
export interface QuickAccessItem {
  title: string
  icon: React.ComponentType<{ className?: string }>
  url: string
  description: string
}
