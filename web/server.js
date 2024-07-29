// server.js
const express = require('express');
const axios = require('axios');
const app = express();
const port = 3000;
const cors = require('cors');

app.use(cors());
app.get('/api/scoreboard/players', async (req, res) => {
    try {
        const response = await axios.get('https://library.m0unt41n.ch/api/v1/scoreboard/players');
        res.json(response.data);
    } catch (error) {
        res.status(500).send('Error fetching data');
    }
});

app.get('/api/ctf', async (req, res) => {
    try {
        const response = await axios.get('https://library.m0unt41n.ch/api/v1/ctf');
        res.json(response.data);
    } catch (error) {
        res.status(500).send('Error fetching data');
    }
});

app.get('/api/players', async (req, res) => {
    try {
        const response = await axios.get('https://library.m0unt41n.ch/api/v1/players');
        res.json(response.data);
    } catch (error) {
        res.status(500).send('Error fetching data');
    }
});

app.listen(port, () => {
    console.log(`Proxy server running at http://localhost:${port}`);
});

