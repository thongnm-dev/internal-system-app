<script setup lang="ts">
import Button from "primevue/button";

withDefaults(
  defineProps<{
    icon: string;
    severity?: "secondary" | "danger" | "success" | "warning" | "info" | "contrast";
  }>(),
  { severity: "secondary" },
);
</script>

<template>
  <Button :icon="icon" :severity="severity" text rounded size="small" />
</template>
