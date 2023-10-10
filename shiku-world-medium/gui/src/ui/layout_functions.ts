import { defineAsyncComponent, UnwrapRef } from "vue";
import { DataContext, ExtractionObject, ResponsiveValue } from "@/ui/index";
import { DisplayInstance } from "vuetify";

const compute_input_value = (
  extraction_object: ExtractionObject,
  context: DataContext
): any | null => {
  return extract_data(
    extraction_object.key.split("."),
    context[extraction_object.store]
  );
};

export const use_layout_functions = () => ({
  get_dynamic_component: (componentName: string) => {
    return defineAsyncComponent(
      () => import(/* @vite-ignore */ `../components/${componentName}.vue`)
    );
  },
  calc_responsive_value: <T>(
    responsive_value: ResponsiveValue<T>,
    display_instance: UnwrapRef<DisplayInstance>
  ) => {
    const rv = responsive_value;
    if (display_instance.xxl || display_instance.xl) {
      return rv.xl || rv.lg || rv.md || rv.sm || rv.xs;
    }

    if (display_instance.lg) {
      return rv.lg || rv.md || rv.sm || rv.xs;
    }

    if (display_instance.md) {
      return rv.md || rv.sm || rv.xs;
    }

    if (display_instance.sm) {
      return rv.sm || rv.xs;
    }

    if (display_instance.xs) {
      return rv.xs;
    }

    throw Error("Could not extract correct display value for breakpoint.");
  },
  compute_input_value,
  compute_input_values: (
    input_map: { [key: string]: ExtractionObject },
    context: DataContext
  ): { [key: string]: any } | null => {
    if (!input_map) {
      return null;
    }

    const computed_input: { [key: string]: any } = {};
    for (const [key, extraction_object] of Object.entries(input_map)) {
      computed_input[key] = compute_input_value(extraction_object, context);
    }

    return computed_input;
  },
});

const extract_data = (path: string[], data: any): any => {
  const key = path.shift();

  if (!key || !data || data[key] === undefined) {
    return null;
  }

  if (path.length == 0) {
    return data ? data[key] : null;
  }

  return extract_data(path, data[key]);
};
