<template>
  <div class="collision-shape">
    <svg></svg>
  </div>
</template>
<script lang="ts" setup>
import { onMounted, ref, toRefs, watch } from "vue";
import * as d3 from "d3";
import { CollisionShape } from "@/editor/blueprints/CollisionShape";
import { match, P } from "ts-pattern";
const props = defineProps<{
  width: number;
  height: number;
  collision_shape: CollisionShape;
}>();
const fill_color = "rgba(190,0,255,0.55)";
const stroke_color = "rgb(0,0,0)";
const { width, height, collision_shape } = toRefs(props);
type SvgType = d3.Selection<d3.BaseType, unknown, HTMLElement, never>;
const svg = ref<SvgType | null>(null);
onMounted(() => {
  svg.value = d3
    .select(".collision-shape svg")
    .attr("width", width.value)
    .attr("height", height.value);
  render_collision_shape();
});

watch(collision_shape, () => {
  render_collision_shape();
});

function render_collision_shape() {
  if (!svg.value) {
    return;
  }

  svg.value.selectAll(".polygon").remove();
  match(collision_shape.value)
    .with({ Polygon: P.select() }, (vertices) =>
      draw_from_polygon(svg.value!, vertices),
    )
    .with({ Rectangle: P.select() }, (rect_definition) =>
      draw_from_rectangle(svg.value!, rect_definition),
    )
    .with({ Circle: P.select() }, (circle_definition) =>
      draw_from_circle(svg.value!, circle_definition),
    )
    .exhaustive();
}

function draw_from_polygon(svg: SvgType, vertices: [number, number][]) {
  const line = d3
    .line()
    .x((d) => {
      return d[0];
    })
    .y((d) => {
      return d[1];
    })
    .curve(d3.curveLinearClosed);

  svg
    .append("path")
    .attr("class", "polygon")
    .attr("d", line(vertices))
    .attr("fill", fill_color)
    .attr("stroke", stroke_color);
}

function draw_from_rectangle(
  svg: SvgType,
  rect_definition: [number, number, number, number],
) {
  svg
    .append("rect")
    .attr("class", "polygon")
    .attr("x", rect_definition[0])
    .attr("y", rect_definition[1])
    .attr("width", rect_definition[2])
    .attr("height", rect_definition[3])
    .attr("fill", fill_color)
    .attr("stroke", stroke_color);
}

function draw_from_circle(
  svg: SvgType,
  circle_definition: [number, number, number],
) {
  svg
    .append("circle")
    .attr("class", "polygon")
    .attr("cx", circle_definition[0])
    .attr("cy", circle_definition[1])
    .attr("r", circle_definition[2])
    .attr("fill", fill_color)
    .attr("stroke", stroke_color);
}
</script>
<style>
.collision-shape {
}
</style>
