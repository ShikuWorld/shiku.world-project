import Fastify, { FastifyReply, FastifyRequest } from 'fastify';
import { configDotenv } from 'dotenv';
import { v4 as uuidv4 } from 'uuid';
import BetterSqlite3 from 'better-sqlite3';

configDotenv();

const fastify = Fastify({ logger: true });

const db = new BetterSqlite3('./database.db');
const API_KEY = 'rE@2#xA3GT&croWDPbtZhARE5KCnx@bQ2DnBB';
function initializeDatabase() {
  db.exec(`
    CREATE TABLE IF NOT EXISTS sessions (
      id TEXT PRIMARY KEY,
      text_data TEXT NOT NULL
    );
  `);
}

initializeDatabase();

fastify.setErrorHandler(function (error, request, reply) {
  // Log error
  this.log.error(error);
  this.log.error(request);
  console.error(request);
  // Send error response
  reply.status(500).send({ ok: false });
});

async function verifyApiKey(request: FastifyRequest, reply: FastifyReply) {
  const apiKey = request.headers['x-api-key'];
  if (apiKey !== API_KEY) {
    reply.status(401).send({ error: 'Unauthorized' });
    throw new Error('Unauthorized');
  }
}

// Register the hook
fastify.addHook('preHandler', verifyApiKey);
fastify.post('/session', async (request, reply) => {
  const sessionId = uuidv4();
  const initialText = request?.body?.text; // Extract 'text' from the JSON request body
  if (!initialText) {
    return reply
      .status(400)
      .send({ error: 'Please provide a text field in the JSON request body.' });
  }
  const insert = db.prepare(
    'INSERT INTO sessions (id, text_data) VALUES (?, ?)'
  );
  insert.run(sessionId, initialText);
  return reply.send({ id: sessionId }); // Return JSON object with 'id'
});

fastify.get('/session/:id', async (request, reply) => {
  const { id } = request.params;
  const select = db.prepare('SELECT text_data FROM sessions WHERE id = ?');
  const row = select.get(id);
  if (!row || !row.text_data) {
    return reply.status(404).send({ error: 'Session not found' });
  }
  return reply.send({ text: row.text_data }); // Return JSON object with 'text'
});

fastify.patch('/session/:id', async (request, reply) => {
  const { id } = request.params;
  const newText = request?.body?.text; // Extract 'text' from the JSON request body
  if (!newText) {
    return reply
      .status(400)
      .send({ error: 'Please provide a text field in the JSON request body.' });
  }
  const update = db.prepare(
    'UPDATE sessions SET text_data = text_data || ? WHERE id = ?'
  );
  const result = update.run(`${newText}\n`, id);
  if (result.changes == 0) {
    return reply.status(404).send({ error: 'Session not found' });
  }

  const select = db.prepare('SELECT text_data FROM sessions WHERE id = ?');
  const row = select.get(id);
  if (!row || !row.text_data) {
    return reply.status(404).send({ error: 'Session not found' });
  }
  return reply.send({ text: row.text_data });
});

fastify.listen({ host: '0.0.0.0', port: 3000 }, (err, _address) => {
  if (err) throw err;
  // Server is now listening on ${address}
});
process.on('SIGTERM', async function onSigterm() {
  await fastify.close();
});
