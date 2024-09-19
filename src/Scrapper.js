const express = require('express');
const { scrapeWebsite, loadCookies } = require('./scraper');

const app = express();
const port = 3000;

app.use(express.json());

let scrapedData = null;

app.get('/api/status', (req, res) => {
  res.json({ status: scrapedData ? 'ready' : 'not ready' });
});

app.post('/api/request', async (req, res) => {
  const { url } = req.body;
  if (!url) {
    return res.status(400).json({ error: 'URL is required' });
  }

  scrapedData = await scrapeWebsite(url);
  res.json({ message: 'Data scraping initiated' });
});

app.get('/api/data', (req, res) => {
  if (scrapedData) {
    res.json(scrapedData);
  } else {
    res.status(404).json({ error: 'No data available' });
  }
});

app.get('/api/cookies', async (req, res) => {
  const cookies = await loadCookies();
  if (cookies) {
    res.json(cookies);
  } else {
    res.status(404).json({ error: 'No cookies available' });
  }
});

app.listen(port, () => {
  console.log(`Server running at http://localhost:${port}`);
});


