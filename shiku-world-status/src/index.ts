import Fastify from 'fastify';
import fastifyStatic from '@fastify/static';
import path from 'path';
import axios from 'axios';
import cors from '@fastify/cors';
import { configDotenv } from 'dotenv';

configDotenv();

const fastify = Fastify({
  logger: true
});
fastify.register(cors, {
  origin: '*'
});
fastify.register(fastifyStatic, {
  root: path.join(__dirname, 'public'),
  prefix: '/public/'
});
fastify.get('/', async (_request, reply) => {
  return reply.sendFile('./index.html', __dirname);
});

type DoorStatusCheck =
  | { type: 'open' }
  | { type: 'lightsOn' }
  | { type: 'lightsOut' }
  | { type: 'urlNotConfigured' }
  | { type: 'unknownError'; error: Error };

async function getDoorStatus(
  statusUrl: string | undefined
): Promise<DoorStatusCheck> {
  try {
    if (!statusUrl) {
      return { type: 'urlNotConfigured' };
    }
    const response = await axios.get<boolean>(statusUrl);
    if (response.status === 218) {
      return { type: 'lightsOut' };
    }
    return response.data ? { type: 'open' } : { type: 'lightsOn' };
  } catch (e) {
    if (e instanceof Error) {
      if (e.message.includes('ECONNREFUSED')) {
        return { type: 'lightsOut' };
      }
      if (e.message.includes('502')) {
        return { type: 'lightsOut' };
      }
      return { type: 'unknownError', error: e };
    }

    return {
      type: 'unknownError',
      error: new Error(`Unknown error occurred ${e}`)
    };
  }
}
fastify.get('/main-door-status', async (request, reply) => {
  return reply.send(await getDoorStatus(process.env.GET_MAIN_DOOR_STATUS_URL));
});

fastify.get('/back-door-status', async (request, reply) => {
  return reply.send(await getDoorStatus(process.env.GET_BACK_DOOR_STATUS_URL));
});

fastify.listen({ host: '0.0.0.0', port: 3000 }, (err, _address) => {
  if (err) throw err;
  // Server is now listening on ${address}
});
