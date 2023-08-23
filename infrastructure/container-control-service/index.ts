import {WebSocketServer} from 'ws';
import {readContainers, useContainerActions} from "./container-actions";
import {match, P} from "ts-pattern";

type Command = {name: 'list'} | {name: 'start', containerName: string} | {name: 'stop', containerName: string};
const startServer = async () => {
  await new Promise(resolve => setTimeout(resolve, 10000));
  const containers = await readContainers('podman');
  const containerNames = containers.map(c => c.name);
  const {startContainer, stopContainer, getContainerStatuses} = useContainerActions(containers, 'podman');

  const wss = new WebSocketServer({ port: 4088 });

  wss.on('connection', (ws) => {
    console.log('WebSocket client connected');

    const interval = setInterval(async () => {
      try {
        const statusObj = Object.fromEntries(await getContainerStatuses(containerNames));
        ws.send(JSON.stringify(statusObj));
      } catch(e) {
        console.error(e);
      }
    }, 2500);

    ws.on('close', () => {
      console.log('WebSocket client disconnected');
      clearInterval(interval);
    });

    ws.on('message', async (message) => {
      try {
        const command = JSON.parse(message.toString()) as Command;

        match(command)
          .with({name: 'list'}, () => ws.send(JSON.stringify(containerNames)))
          .with({name: 'start', containerName: P.select()}, async (containerName) => {
            if (!containerNames.includes(containerName)) {
              ws.send(JSON.stringify({"error": `Container "${containerName}" does not exist.`}));
              return;
            }
            try {
              await startContainer(containerName);
            } catch (e) {
              ws.send(JSON.stringify({"error": `Error while trying to start "${containerName}". ${e.message ? e.message : e}`}));
            }
          })
          .with({name: 'stop', containerName: P.select()}, async (containerName) => {
            if (!containerNames.includes(containerName)) {
              ws.send(JSON.stringify({"error": `Container "${containerName}" does not exist.`}));
              return;
            }
            try {
              await stopContainer(containerName);
            } catch (e) {
              ws.send(JSON.stringify({"error": `Error while trying to stop "${containerName}". ${e.message ? e.message : e}`}));
            }
      })
          .otherwise((invalid_input) => {
            const _compileTimeExhaustiveCheck: never = invalid_input;
            ws.send(JSON.stringify({"error": `Invalid request ${_compileTimeExhaustiveCheck}, please provide name and command params as json object.`}));
          });
      } catch (e) {
        ws.send(JSON.stringify({"error": `Invalid request, please provide name and command params as json object.`}));
      }
    });
  });
};

startServer();
