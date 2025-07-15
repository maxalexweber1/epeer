import { useEffect, useState } from 'react'
import axios from 'axios'

export default function NetworkOverview() {
    const [summary, setSummary] = useState<{ produced: number; consumed: number }>({
        produced: 0,
        consumed: 0,
    })

    //    useEffect(() => {
    //        const fetch = async () => {
    //            const res = await axios.get('http://localhost:5000/api/v1/messages?topic=energy_events')
    //            const data = res.data.items
    //                .map((msg: any) => msg.data?.value)
    //                .filter(Boolean)

    //            const latestPerNode: Record<string, any> = {}
    //            data.forEach((e: any) => {
    //                if (!e.participant) return
    //                latestPerNode[e.participant] = e
    //            })

    //            const totalProduced = Object.values(latestPerNode).reduce(
    //                (sum: number, d: any) => sum + (d.energy_produced || 0),
    //                0
    //            )
    //            const totalConsumed = Object.values(latestPerNode).reduce(
    //                (sum: number, d: any) => sum + (d.energy_consumed || 0),
    //                0
    //            )

    //            setSummary({ produced: totalProduced, consumed: totalConsumed })
    //        }

    //        fetch()
    //        const interval = setInterval(fetch, 5000)
    //        return () => clearInterval(interval)
    //    }, [])

    return (
        <div>
            <h2 className="text-xl font-bold">🌍 Netzwerk Übersicht</h2>
            <p>🔋 Gesamt-Produktion: {summary.produced} kWh</p>
            <p>⚡ Gesamt-Verbrauch: {summary.consumed} kWh</p>
        </div>
    )
}
