import { useEffect, useState } from 'react'
import axios from 'axios'

type Props = {
    participant: string
}

export default function EnergyDashboard({ participant }: Props) {
    const [data, setData] = useState<any>()

    // useEffect(() => {
    //     const fetch = async () => {
    //         const res = await axios.get(`http://localhost:5000/api/v1/messages?topic=energy_events`)
    //         const items = res.data.items
    //             .map((msg: any) => ({ ...msg.data?.value, timestamp: Date.parse(msg.created) }))
    //             .filter((e) => e.participant === participant)
    //             .sort((a, b) => b.timestamp - a.timestamp)
    //
    //          setData(items[0])
    //      }

    // fetch()
    // const interval = setInterval(fetch, 5000)
    // return () => clearInterval(interval)
    // }, [participant])

    if (!data) return <p>Lade {participant} ...</p>

    return (
        <div>
            <h2 className="text-xl font-bold">{participant}</h2>
            <p>🔋 Produktion: {data.energy_produced} kWh</p>
            <p>⚡ Verbrauch: {data.energy_consumed} kWh</p>
            <p>📢 Event: {data.event || '–'}</p>
        </div>
    )
}
