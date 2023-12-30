export const use_medium_api = (): typeof window.medium =>
  window.medium
    ? window.medium
    : {
        twitch_login: () => Promise.resolve(),
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        communication_state: {} as any,
        set_camera_iso: (instance_id, world_id, iso) => {
          console.log(
            `set_camera_iso not implemented ${instance_id}, ${world_id}, ${iso}`,
          );
        },
        set_camera_zoom: (instance_id, world_id) => {
          console.log(
            `set_camera_zoom not implemented ${instance_id}, ${world_id}`,
          );
        },
        get_camera_iso: (instance_id, world_id) => {
          console.log(
            `get_camera_iso not implemented ${instance_id}, ${world_id}`,
          );
          return { x: 0, y: 0, rotation: 0 };
        },
        get_camera_zoom: (instance_id, world_id) => {
          console.log(
            `get_camera_zoom not implemented ${instance_id}, ${world_id}`,
          );
          return 0;
        },
        swap_main_render_instance: () => {},
      };
