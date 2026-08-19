import express from "express";
import bodyParser from "body-parser";

import mockData from "./data.json" with {type: "json"};

const app = express();
const port = 3000;

const SECS_IN_DAY = 86_400;

app.use(bodyParser.json());

function generateRandomId() {
  const chars = "ABCDEF0123456789";
  let id = "";
  for (let i = 0; i < 12; i++) {
    id += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return id;
}

// GET /api/mapping
app.get("/api/mappings", (req, res) => {
  res.json(Object.keys(mockData.mapping));
});

app.get("/api/mapping", (req, res) => {
  const id = req.query["id"]
  const mapping = mockData.mapping[id];

  if (mapping) {
    res.json(mapping);
  } else {
    res.status(404).send();
  }
});

// POST /api/mapping
app.post("/api/mapping", (req, res) => {
  const { id, name } = req.body;

  if (!id || !name || !name.first || !name.last) {
    return res.status(400).json({ error: "Invalid request body" });
  }

  // Check if ID already exists
  const existing = mappings.find((entry) => entry[0] === id);
  if (existing) {
    return res.status(409).json({ error: "ID already exists" });
  }

  // Add new mapping
  mockData.mappings.push([id, name]);

  res.status(201).send("");
});

app.get("/api/day", (req, res) => {
  let day;

  if (req.query.day) {
    day = parseInt(req.query.day, 10);
  } else if (req.query.timestamp) {
    let ts = parseInt(req.query.timestamp, 10);
    day = ts / SECS_IN_DAY;
  } else {
    return res.status(400).json({ error: "Missing or invalid 'day' parameter" });
  }

  if (isNaN(day)) {
    return res.status(400).json({ error: "Missing or invalid 'day' parameter" });
  }

  let foundDay = mockData.days.find(e => e.date == day);

  if (!foundDay) {
    return res.status(404).send("Not found");
  }

  res.status(200).json(foundDay);
});

app.get("/api/days", (req, res) => {

  let qFrom = parseInt(req.query.from) / SECS_IN_DAY;
  let qTo = parseInt(req.query.to) / SECS_IN_DAY;

  let days = mockData.days.filter(e => e.date >= qFrom && e.date <= qTo).map(e => e.date);

  res.status(200).json(days);
});

// SSE route: /api/idevent
app.get("/api/idevent", (req, res) => {
  // Set headers for SSE
  res.setHeader("Content-Type", "text/event-stream");
  res.setHeader("Cache-Control", "no-cache");
  res.setHeader("Connection", "keep-alive");

  res.flushHeaders(); // flush the headers to establish SSE connection

  // Send initial event
  const sendEvent = () => {
    const id = generateRandomId();
    res.write(`event: msg\ndata: ${id}\n\n`);
  };

  // Send immediately and then every 10 seconds
  sendEvent();
  const interval = setInterval(sendEvent, 10000);

  // When client closes connection, stop interval
  req.on("close", () => {
    clearInterval(interval);
    res.end();
  });
});

// GET /api/time
app.get("/api/time", (req, res) => {
  const now = Date.now()
  res.send((now / 1000).toString())
});

// Start the server
app.listen(port, () => {
  console.log(`Mock API server running at http://localhost:${port}`);
});
