import { UiStore } from "@/editor/stores/ui";
import { KeysOfType, Paths } from "@/editor/utils";
import { CurrentModuleStore } from "@/editor/stores/current-module";
import { HLayout } from "@/editor/components/HLayout.vue";
import { VLayout } from "@/editor/components/VLayout.vue";
import { MainMenu } from "@/editor/components/MainMenu.vue";
import { LoginMenu } from "@/editor/components/LoginMenu.vue";
import { TutorialComponent } from "@/editor/components/TutorialComponent.vue";
import { NumberStats } from "@/editor/components/NumberStats.vue";
import { ImageHoverMap } from "@/editor/components/ImageHoverMap.vue";

export * from "./layout_functions";

export type DataContext = {
  current_module: CurrentModuleStore;
  ui: Omit<UiStore, "current_menu">;
};

export type ExtractionObject =
  | {
      store: "current_module";
      key: string;
    }
  | { store: "ui"; key: Paths<Omit<UiStore, "current_menu">> };

type VueComponentDefRaw<C extends { name: string }> = {
  name: C["name"];
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  input: C extends { input: any }
    ? {
        [key in keyof C["input"]]: ExtractionObject;
      }
    : undefined;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  config: C extends { config: any } ? C["config"] : undefined;
};

export type VueComponentDef<C extends { name: string; id?: string }> = Omit<
  VueComponentDefRaw<C>,
  KeysOfType<VueComponentDefRaw<C>, undefined>
> &
  Partial<
    Pick<VueComponentDefRaw<C>, KeysOfType<VueComponentDefRaw<C>, undefined>>
  >;

export type ComponentConfig =
  | VueComponentDef<NumberStats>
  | VueComponentDef<HLayout>
  | VueComponentDef<MainMenu>
  | VueComponentDef<VLayout>
  | VueComponentDef<LoginMenu>
  | VueComponentDef<TutorialComponent>
  | VueComponentDef<ImageHoverMap>;

export interface ResponsiveValue<T> {
  xs: T;
  sm?: T;
  md?: T;
  lg?: T;
  xl?: T;
}
