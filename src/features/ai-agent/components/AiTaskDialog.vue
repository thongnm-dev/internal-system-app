<script setup lang="ts">
import { ref, watch } from "vue";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import Select from "primevue/select";
import { TASK_CATEGORY_OPTIONS } from "@/_/types/ai-task";
import type { AiTaskCategory, AiTaskResult } from "@/tauri/commands/ai-task";

export type TaskDialogPayload = {
  task_cd: string;
  task_name: string;
  category: AiTaskCategory;
  is_complete: boolean;
};

const props = defineProps<{
  visible: boolean;
  task?: AiTaskResult | null;
  loading?: boolean;
}>();

const emit = defineEmits<{
  "update:visible": [value: boolean];
  save: [payload: TaskDialogPayload];
}>();

const isEdit = ref(false);
const taskCd = ref("");
const taskName = ref("");
const taskCategory = ref<AiTaskCategory>("other");
const isComplete = ref(false);

watch(
  () => props.visible,
  (open) => {
    if (!open) return;
    if (props.task) {
      isEdit.value = true;
      taskCd.value = props.task.task_cd;
      taskName.value = props.task.task_name;
      taskCategory.value = props.task.category as AiTaskCategory;
      isComplete.value = props.task.is_complete;
    } else {
      isEdit.value = false;
      taskCd.value = "";
      taskName.value = "";
      taskCategory.value = "other";
      isComplete.value = false;
    }
  },
);

function handleSave() {
  emit("save", {
    task_cd: taskCd.value.trim(),
    task_name: taskName.value.trim(),
    category: taskCategory.value,
    is_complete: isComplete.value,
  });
}
</script>

<template>
  <Dialog
    :visible="props.visible"
    class="w-full max-w-md rounded-lg bg-panel shadow-xl"
    :closable="true"
    modal
    @update:visible="emit('update:visible', $event)"
  >
    <template #header>
      <h3 class="font-bold text-ink">{{ isEdit ? "Edit Task" : "New Task" }}</h3>
    </template>

    <div class="space-y-4">
      <label class="block">
        <span class="text-xs font-bold text-muted">Task Code <span class="text-red-500">*</span></span>
        <InputText v-model="taskCd" class="mt-1 w-full" placeholder="e.g. SCR-001" autofocus />
      </label>
      <label class="block">
        <span class="text-xs font-bold text-muted">Task Name</span>
        <InputText v-model="taskName" class="mt-1 w-full" placeholder="Brief name of the task" />
      </label>
      <label class="block">
        <span class="text-xs font-bold text-muted">Category</span>
        <Select
          v-model="taskCategory"
          :options="TASK_CATEGORY_OPTIONS"
          option-label="label"
          option-value="value"
          class="mt-1 w-full"
        />
      </label>
      <div v-if="isEdit" class="flex items-center gap-2">
        <Checkbox
          :model-value="isComplete"
          binary
          @change="isComplete = !isComplete"
        />
        <span class="text-sm text-ink">Mark as complete</span>
      </div>
    </div>

    <template #footer>
      <div class="flex items-center justify-end gap-2">
        <Button label="Cancel" severity="secondary" @click="emit('update:visible', false)" />
        <Button
          :label="isEdit ? 'Save' : 'Create'"
          :disabled="!taskCd.trim()"
          :loading="props.loading"
          @click="handleSave"
        />
      </div>
    </template>
  </Dialog>
</template>
