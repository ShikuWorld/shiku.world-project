import { UiStore } from "@/stores/ui";
import { KeysOfType, Paths } from "@/utils";
import { CurrentModuleStore } from "@/stores/current-module";
import { HLayout } from "@/components/HLayout.vue";
import { VLayout } from "@/components/VLayout.vue";
import { MainMenu } from "@/components/MainMenu.vue";
import { LoginMenu } from "@/components/LoginMenu.vue";
import { TutorialComponent } from "@/components/TutorialComponent.vue";
import { NumberStats } from "@/components/NumberStats.vue";
import { ImageHoverMap } from "@/components/ImageHoverMap.vue";

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
  input: C extends { input: any }
    ? {
        [key in keyof C["input"]]: ExtractionObject;
      }
    : undefined;
  config: C extends { config: {} } ? C["config"] : undefined;
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
