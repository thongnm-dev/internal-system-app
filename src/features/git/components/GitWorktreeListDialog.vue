<script setup lang="ts">
import { ref, watch } from "vue";
import Button from "primevue/button";
import Dialog from "primevue/dialog";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });
const emit = defineEmits<{ "create-worktree": [] }>();

const isMaximized = ref(false);

watch(visible, (v) => {
  if (v) {
    isMaximized.value = false;
    props.git.loadWorktrees();
  }
});

function requestCreate() {
  visible.value = false;
  emit("create-worktree");
}
</script>

<template>
  <Dialog
    v-model:visible="visible"
    modal
    maximizable
    header="Worktrees"
    :style="{ width: '620px' }"
    @maximize="isMaximized = true"
    @unmaximize="isMaximized = false"
  >
    <div
      class="flex flex-col gap-2 overflow-y-auto"
      :class="isMaximized ? 'h-[calc(100vh-260px)]' : 'max-h-[420px]'"
    >
      <div
        v-for="w in git.worktrees.value"
        :key="w.path"
        class="flex items-center gap-2 rounded-md border border-divider px-3 py-2"
      >
        <i class="pi shrink-0 text-sm" :class="w.is_current ? 'pi-check-circle text-brand' : 'pi-folder text-muted'" />
        <div class="min-w-0 flex-1">
          <p class="truncate font-mono text-xs text-ink">{{ w.path }}</p>
          <p class="text-[11px] text-muted">
            <span v-if="w.is_bare">bare</span>
            <span v-else-if="w.is_detached">detached @ {{ w.head.slice(0, 7) }}</span>
            <span v-else>{{ w.branch }}</span>
            <span v-if="w.is_current" class="ml-1 text-brand">· đang mở</span>
          </p>
        </div>
        <Button
          v-if="!w.is_current"
          size="small"
          outlined
          severity="secondary"
          class="shrink-0"
          title="Mở worktree này"
          @click="visible = false; git.openPathAsRepo(w.path)"
        >
          <i class="pi pi-external-link" />
        </Button>
        <Button
          v-if="!w.is_current && !w.is_bare"
          size="small"
          outlined
          severity="danger"
          class="shrink-0"
          title="Gỡ worktree"
          @click="git.worktreeRemove(w.path, false)"
        >
          <i class="pi pi-trash" />
        </Button>
      </div>
      <div v-if="!git.worktrees.value.length" class="p-4 text-center text-sm text-muted">
        Chưa có worktree nào.
      </div>
    </div>
    <template #footer>
      <DialogFooter
        cancel-label="Đóng"
        confirm-label="Tạo worktree"
        confirm-icon="pi pi-plus"
        @cancel="visible = false"
        @confirm="requestCreate"
      />
    </template>
  </Dialog>
</template>
