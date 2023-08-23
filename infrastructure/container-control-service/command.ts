import { Server as WebSocketServer, WebSocket } from 'ws';
import { spawn } from 'child_process';
import { createServer } from 'http';

const remoteHost = 'your_remote_host';
const remotePort = 8080; // Or the port that your remote WebSocket server is running on
const privateKey = '/path/to/your/private/key';

const ssh = spawn('ssh', [
  '-i', privateKey, // Path to the private key file
  '-L', '4088:localhost:' + remotePort, // Local port to listen on and remote host/port to forward traffic to
  'your_user@' + remoteHost, // Your username and the remote hostname
  '-N' // Disable terminal mode
]);

ssh.stdout.on('data', (data) => {
  console.log(`stdout: ${data}`);
});

ssh.stderr.on('data', (data) => {
  console.error(`stderr: ${data}`);
});

ssh.on('close', (code) => {
  console.log(`child process exited with code ${code}`);
});

const server = createServer();

const wss = new WebSocketServer({ server });

wss.on('connection', (ws) => {
  const remoteWs = new WebSocket(`ws://localhost:4088`);
  remoteWs.on('open', () => {
    ws.on('message', (message) => {
      remoteWs.send(message);
    });

    ws.on('close', () => {
      remoteWs.close();
    });
  });

  remoteWs.on('message', (message) => {
    ws.send(message);
  });
});

server.listen(4089, () => {
  console.log('WebSocket server listening on port 4089');
});
