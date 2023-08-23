import { spawn } from 'child_process';

export interface Container {
  name: string;
}

export const readContainers = (command: string): Promise<Container[]> => {
  return new Promise((resolve, reject) => {
    const daemon = spawn(command, ['ps', '-a', '--format', '"{{.Names}}"']);
    let containerList: Container[] = [];
    const statusMap = new Map<string, boolean>();
    daemon.stdout.on('data', (data) => {
      containerList = data.toString()
        .replaceAll('\n', ' ')
        .replaceAll('  ', ' ')
        .replaceAll('"', '')
        .trim()
        .split(' ').map(
          c => ({name: c})
        );
    });
    console.log(statusMap);

    daemon.stderr.on('data', (data) => {
      reject(data.toString());
    });

    daemon.on('close', (code) => {
      if (code === 0) {
        resolve(containerList);
      } else {
        reject(`podman exited with code ${code}`);
      }
    });
  });
};

export const useContainerActions = (containers: Container[], engine?: string) => {
  const command = engine ? engine : 'podman';
  return {
    getContainerStatuses: (_names: string[]): Promise<Map<string, boolean>> => {
      return new Promise((resolve, reject) => {
        const daemon = spawn(command, ['ps', '--format', '"{{.Names}}"']);

        const statusMap = new Map<string, boolean>();
        daemon.stdout.on('data', (data) => {
          const online_container_list = data.toString()
            .replaceAll('\n', ' ')
            .replaceAll('  ', ' ')
            .replaceAll('"', '')
            .trim()
            .split(' ');

          for (const container of containers) {
            if (online_container_list.includes(container.name)) {
              statusMap.set(container.name, true);
            }
          }
        });

        daemon.stderr.on('data', (data) => {
          reject(data.toString());
        });

        daemon.on('close', (code) => {
          if (code === 0) {
            resolve(statusMap);
          } else {
            reject(`podman exited with code ${code}`);
          }
        });
      });
    },
    startContainer: (name: string): Promise<void> => {
      return new Promise((resolve, reject) => {
        if (!containers.find((container) => container.name === name)) {
          reject(`Container '${name}' not found`);
        }

        const systemctl = spawn('systemctl', ['start', `${command}-${name}.service`], {timeout: 5000});
        systemctl.stderr.on('data', (data) => {
          reject(data.toString());
        });

        systemctl.on('close', (code) => {
          if (code === 0) {
            resolve();
          } else {
            reject(`podman exited with code ${code}`);
          }
        });
      });
    },
    stopContainer: (name: string): Promise<void> => {
      return new Promise((resolve, reject) => {
        if (!containers.find((container) => container.name === name)) {
          reject(`Container '${name}' not found`);
        }

        const podman = spawn('systemctl', ['stop', `${command}-${name}.service`], {timeout: 5000});

        podman.stdout.on('data', (data) => {
          console.log(`stdout: ${data}`);
        });

        podman.stderr.on('data', (data) => {
          console.error(`stderr: ${data}`);
          reject(data.toString());
        });

        podman.on('close', (code) => {
          if (code === 0) {
            resolve();
          } else {
            reject(`podman exited with code ${code}`);
          }
        });
      });
    }
  }
}
