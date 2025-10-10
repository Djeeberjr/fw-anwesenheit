import express from "express";
import bodyParser from "body-parser";

const app = express();
const port = 3000;

app.use(bodyParser.json());

let mappings = [
  [
    "123456789ABC",
    {
      first: "Feuerwehrman",
      last: "Sam",
    },
  ],
];

function generateRandomId() {
  const chars = "ABCDEF0123456789";
  let id = "";
  for (let i = 0; i < 12; i++) {
    id += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return id;
}

// GET /api/mapping
app.get("/api/mapping", (req, res) => {
  res.json(mappings);
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
  mappings.push([id, name]);

  res.status(201).send("");
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
    res.write(`data: ${id}\n\n`);
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

// Start the server
app.listen(port, () => {
  console.log(`Mock API server running at http://localhost:${port}`);
});
