<script setup lang="ts">
import Button from "primevue/button";

withDefaults(
  defineProps<{
    active: boolean;
    icon?: string;
    label?: string;
    variant?: "segment" | "filter";
    disabled?: boolean;
  }>(),
  { variant: "segment" },
);

defineEmits<{
  click: [];
}>();
</script>

<template>
  <Button
    :class="
      variant === 'filter'
        ? active
          ? 'filter-chip-active'
          : 'filter-chip-inactive'
        : active
          ? 'toggle-chip-active'
          : 'toggle-chip-inactive'
    "
    unstyled
    :disabled="disabled"
    @click="$emit('click')"
  >
    <i v-if="icon" :class="['pi', icon, 'shrink-0 text-xs']" />
    <span v-if="label" class="truncate">{{ label }}</span>
    <slot v-else />
  </Button>
</template>
