<template>
  <v-chip class="modules-editor-inner__socket" v-for="slot in slots"
    >{{ slot.name }}
    <v-dialog width="500">
      <template v-slot:activator="{ props }">
        <v-icon
          :icon="mdiTrashCan"
          color="error"
          v-bind="props"
          size="x-small"
        ></v-icon>
      </template>

      <template v-slot:default="{ isActive }">
        <v-card :title="`Delete ${slot.name}?`">
          <v-card-actions>
            <v-spacer></v-spacer>

            <v-btn
              text="Yes!"
              @click="
                emit('delete', slot.name);
                isActive.value = false;
              "
            ></v-btn>
          </v-card-actions>
        </v-card>
      </template>
    </v-dialog>
  </v-chip>
</template>
<script lang="ts" setup>
import { IOPoint } from "@/editor/blueprints/IOPoint";
import { mdiTrashCan } from "@mdi/js";

defineProps<{ slots: IOPoint[] }>();
const emit = defineEmits<{ (e: "delete", value: string): void }>();
</script>
<style>
.modules-editor-inner__socket {
  margin: 10px;
}
</style>
