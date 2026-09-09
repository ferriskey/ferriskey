import { Area, AreaChart, ResponsiveContainer } from 'recharts'

const toneHex = {
  info: '#2379ff',
  success: '#009764',
  amber: '#d97706',
  violet: '#8b5cf6',
  brand: '#e8431a',
  primary: '#c0602a',
} as const

export type ChartTone = keyof typeof toneHex

export function Sparkline({
  data,
  tone = 'info',
  height = 26,
}: {
  data: number[]
  tone?: ChartTone
  height?: number
}) {
  const points = data.map((v, i) => ({ i, v }))
  const color = toneHex[tone]
  const id = `spark-${tone}-${data.join('-')}`

  return (
    <ResponsiveContainer width='100%' height={height}>
      <AreaChart data={points} margin={{ top: 2, right: 0, bottom: 0, left: 0 }}>
        <defs>
          <linearGradient id={id} x1='0' y1='0' x2='0' y2='1'>
            <stop offset='0%' stopColor={color} stopOpacity={0.28} />
            <stop offset='100%' stopColor={color} stopOpacity={0} />
          </linearGradient>
        </defs>
        <Area
          type='monotone'
          dataKey='v'
          stroke={color}
          strokeWidth={2}
          fill={`url(#${id})`}
          isAnimationActive={false}
          dot={false}
        />
      </AreaChart>
    </ResponsiveContainer>
  )
}
