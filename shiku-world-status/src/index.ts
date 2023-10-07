import Fastify from 'fastify';
import fastifyStatic from '@fastify/static';
import path from 'path';

const fastify = Fastify({
  logger: true
});

fastify.register(fastifyStatic, {
  root: path.join(__dirname, 'public'),
  prefix: '/public/'
});
fastify.get('/', async (_request, reply) => {
  return reply.sendFile('./index.html', __dirname);
});

fastify.listen({ host: '0.0.0.0', port: 3000 }, (err, _address) => {
  if (err) throw err;
  // Server is now listening on ${address}
});
