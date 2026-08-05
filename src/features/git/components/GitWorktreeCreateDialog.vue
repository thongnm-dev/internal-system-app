<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import InputGroup from "primevue/inputgroup";
import Select from "primevue/select";
import { open } from "@tauri-apps/plugin-dialog";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });

const wtParent = ref("");
const wtFolder = ref("");
const wtCreateNewBranch = ref(false);
const wtExistingBranch = ref("");
const wtNewBranch = ref("");
const wtOpenAfter = ref(true);

const worktreeBranchOptions = computed(() =>
  props.git.localBranches.value.map((b) => ({ label: b.name, value: b.name })),
);

function resetWorktreeForm() {
  wtParent.value = "";
  wtFolder.value = "";
  wtCreateNewBranch.value = false;
  wtExistingBranch.value = props.git.info.value?.current_branch ?? "";
  wtNewBranch.value = "";
  wtOpenAfter.value = true;
}

watch(visible, (v) => {
  if (v) resetWorktreeForm();
});

async function pickWorktreeParent() {
  const picked = await open({ directory: true, title: "Chọn thư mục cha cho worktree" });
  if (picked && typeof picked === "string") wtParent.value = picked;
}

function joinPath(parent: string, name: string) {
  const sep = parent.includes("\\") ? "\\" : "/";
  return `${parent.replace(/[/\\]+$/, "")}${sep}${name}`;
}

const worktreeCanCreate = computed(() => {
  if (!wtParent.value.trim()) return false;
  return wtCreateNewBranch.value ? !!wtNewBranch.value.trim() : !!wtExistingBranch.value;
});

async function doWorktreeCreate() {
  if (!worktreeCanCreate.value) return;
  const branchRef = wtCreateNewBranch.value ? wtNewBranch.value.trim() : wtExistingBranch.value;
  const defaultFolder = (branchRef.split(/[/\\]/).pop() || "worktree").trim();
  const folder = (wtFolder.value.trim() || defaultFolder) || "worktree";
  const fullPath = joinPath(wtParent.value, folder);
  const created = await props.git.worktreeAdd(
    fullPath,
    wtCreateNewBranch.value ? "" : wtExistingBranch.value,
    wtCreateNewBranch.value ? wtNewBranch.value.trim() : "",
  );
  if (created) {
    visible.value = false;
    if (wtOpenAfter.value) await props.git.openPathAsRepo(created);
  }
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Tạo worktree" :style="{ width: '520px' }">
    <div class="flex flex-col gap-3">
      <div>
        <label class="mb-1 block text-xs font-bold text-muted">Thư mục cha</label>
        <InputGroup class="h-8">
          <InputText :model-value="wtParent" readonly placeholder="Chọn thư mục…" />
          <Button icon="pi pi-folder-open" severity="secondary" outlined title="Chọn thư mục cha" @click="pickWorktreeParent" />
          <Button v-if="wtParent" icon="pi pi-times" severity="danger" text title="Xoá đường dẫn" @click="wtParent = ''" />
        </InputGroup>
      </div>
      <div>
        <label class="mb-1 block text-xs font-bold text-muted">Tên thư mục worktree</label>
        <InputText v-model="wtFolder" placeholder="(mặc định theo tên branch)" class="w-full" />
      </div>
      <div class="flex items-center gap-2">
        <Checkbox v-model="wtCreateNewBranch" binary input-id="wt-new-branch" />
        <label for="wt-new-branch" class="text-sm text-ink">Tạo branch mới (từ HEAD hiện tại)</label>
      </div>
      <div v-if="wtCreateNewBranch">
        <label class="mb-1 block text-xs font-bold text-muted">Tên branch mới</label>
        <InputText v-model="wtNewBranch" placeholder="feature/ten-branch" class="w-full" />
      </div>
      <div v-else>
        <label class="mb-1 block text-xs font-bold text-muted">Branch (đã có)</label>
        <Select
          v-model="wtExistingBranch"
          :options="worktreeBranchOptions"
          option-label="label"
          option-value="value"
          placeholder="Chọn branch…"
          filter
          class="w-full"
        />
      </div>
      <div class="flex items-center gap-2">
        <Checkbox v-model="wtOpenAfter" binary input-id="wt-open-after" />
        <label for="wt-open-after" class="text-sm text-ink">Mở worktree sau khi tạo</label>
      </div>
    </div>
    <template #footer>
      <DialogFooter
        cancel-label="Hủy"
        confirm-label="Tạo worktree"
        confirm-icon="pi pi-clone"
        :confirm-disabled="!worktreeCanCreate || !!git.busyMessage.value"
        @cancel="visible = false"
        @confirm="doWorktreeCreate"
      />
    </template>
  </Dialog>
</template>
