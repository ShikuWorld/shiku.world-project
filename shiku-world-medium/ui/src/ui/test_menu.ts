import { ComponentConfig } from "@/editor/ui/index";

export const test_menu: ComponentConfig = {
  name: "HLayout",
  config: {
    columns: [
      {
        cols: {
          xs: 12,
          sm: 10,
          md: 8,
          lg: 6,
          xl: 5,
        },
        component: {
          name: "MainMenu",
          config: {
            tabs: [
              {
                label: "Stats",
                icon: "mdiBagPersonal",
                component: {
                  name: "ImageHoverMap",
                  config: {
                    height: 200,
                    images: [
                      {
                        state_value: {
                          store: "current_module",
                          key: "data.someState",
                        },
                        default_image_src:
                          "src/assets/emerald_piece_A_1_off.png",
                        pos_x: 20,
                        pos_y: 40,
                        states: [
                          {
                            value: 1,
                            image_src: "src/assets/emerald_piece_A_1.png",
                            hover_text: "Secret in the woods",
                          },
                        ],
                        default_hover_text: "You did it!",
                      },
                    ],
                  },
                },
              },
              {
                label: "Stats",
                icon: "mdiChartBar",
                component: {
                  name: "NumberStats",
                  config: {
                    stats: [
                      {
                        label: "Guests online",
                        number: {
                          store: "current_module",
                          key: "data.current_guest_info.guests_online",
                        },
                      },
                    ],
                  },
                },
              },
              {
                label: "Tutorial",
                icon: "mdiHelp",
                component: {
                  name: "TutorialComponent",
                  config: {
                    tutorials: [
                      {
                        id: "General Info",
                        label: "General",
                        text: "Here are some tutorials on how to play slime adventures with different input methods. To change the input method, click on the controller symbol in the bottom of the screen. Remember that Keyboard inputs are disabled for twitch!",
                      },
                      {
                        id: "Mouse Tutorial",
                        label: "Mouse",
                        pics: [
                          "src/assets/mouse_tutorial/move.png",
                          "src/assets/mouse_tutorial/small_jump.png",
                          "src/assets/mouse_tutorial/high_jump.png",
                          "src/assets/mouse_tutorial/high_directional_jump.png",
                        ],
                      },
                      {
                        id: "Controller Tutorial",
                        label: "Controller",
                        pics: [
                          "src/assets/controller_tutorial/tutorial1_controller.png",
                          "src/assets/controller_tutorial/tutorial2_controller.png",
                          "src/assets/controller_tutorial/tutorial3_controller.png",
                          "src/assets/controller_tutorial/tutorial4_controller.png",
                        ],
                      },
                      {
                        id: "Keyboard Tutorial",
                        label: "Keyboard",
                        pics: [
                          "src/assets/keyboard_tutorial/tutorial1_keyboard.png",
                          "src/assets/keyboard_tutorial/tutorial2_keyboard.png",
                          "src/assets/keyboard_tutorial/tutorial3_keyboard.png",
                          "src/assets/keyboard_tutorial/tutorial4_keyboard.png",
                        ],
                      },
                    ],
                  },
                },
              },
            ],
          },
        },
      },
    ],
  },
};
