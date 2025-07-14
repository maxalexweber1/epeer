// backend/index.js
const express = require('express');
const app = express();

app.get('/health', (_req, res) => res.send('OK'));

const port = 4000;
app.listen(port, () => console.log(`Backend listening on ${port}`));