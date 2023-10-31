<template>
  <div @pointerdown.stop="" @dblclick.stop="" @click="data.onClick">
    {{ data.label }}
  </div>
</template>

<script lang="ts" setup>
defineProps<{ data: { label: string; onClick: () => void } }>();
</script>
