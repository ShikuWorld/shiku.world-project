<template>
  <div class="editor-log">
    <div
      :class="`editor-log__entry editor-log__entry--${level.toLowerCase()}`"
      v-for="[id, _time, level, _location, message] in filtered_logs"
      :key="id"
    >
      {{ message }}
    </div>
  </div>
</template>
<script lang="ts" setup>
import { computed, toRefs } from "vue";
import { LogInfo } from "@/editor/blueprints/LogInfo";
import { match } from "ts-pattern";

const props = defineProps<{
  logs: LogInfo[];
  log_level?: string;
  log_location?: string;
}>();
const { logs, log_level, log_location } = toRefs(props);

const error_levels = ["ERROR"];
const debug_levels = ["ERROR", "DEBUG"];
const trace_levels = ["ERROR", "DEBUG", "TRACE"];

const filtered_logs = computed(() => {
  return logs.value
    .filter(([_id, _time, level, location, _message]) => {
      if (log_level.value) {
        return match(log_level.value)
          .with("ERROR", (): string[] => error_levels)
          .with("DEBUG", (): string[] => debug_levels)
          .with("TRACE", (): string[] => trace_levels)
          .otherwise((): string[] => [])
          .includes(level);
      }
      return !(log_location.value && location !== log_location.value);
    })
    .reverse();
});
</script>
<style lang="scss">
.editor-log {
  position: absolute;
  bottom: 0;
  height: 80px;
  background-color: #1a192b;
  padding: 10px;
  width: 100%;
  display: flex;
  flex-direction: column;
  z-index: 5000;
  pointer-events: all;
  overflow: auto;
}

@keyframes new-log {
  from {
    background-color: rgba(252, 223, 223, 0.75);
  }
  to {
    background-color: rgba(255, 255, 255, 0);
  }
}

.editor-log__entry--error {
  color: #ff6c6c;
  animation-name: new-log;
  animation-duration: 2s;
  font-size: 8pt;
}

.editor-log__entry {
  white-space: nowrap;
}

.editor-nav-right > div {
  height: 100%;
}
</style>
