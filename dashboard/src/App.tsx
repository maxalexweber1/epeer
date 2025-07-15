import { useState, useEffect } from 'react'
import './App.css'
import { Bar } from 'react-chartjs-2';
import axios from 'axios';
import EnergyDashboard from './components/EnergyDashboard';
import NetworkOverview from './components/NetworkOverview';

import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  BarElement,
  Title,
  Tooltip,
  Legend,
} from 'chart.js';

ChartJS.register(
  CategoryScale,
  LinearScale,
  BarElement,
  Title,
  Tooltip,
  Legend
)

type NodeEvent = {
  participant: string
  energy_produced: number
  energy_consumed: number
  event?: string
  timestamp?: number
}


function Cluster({ label }: { label: string }) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: '2rem',
        color: 'white',
        backgroundColor: '#2c3e50',
        boxShadow: 'inset 0 0 0 1px #444',
      }}
    >
      {label}
    </div>
  );
}

function App() {
  return (
    <div
      style={{
        display: 'grid',
        gridTemplateColumns: '1fr 1fr',
        gridTemplateRows: '1fr 1fr',
        width: '100vw',
        height: '100vh',
      }}
    >
      <Cluster label="Cluster 1" />
      <Cluster label="Cluster 2" />
      <Cluster label="Cluster 3" />
      <Cluster label="Cluster 4" />
    </div>
  );
}



export default App
