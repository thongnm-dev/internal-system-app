<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import Select from "primevue/select";
import DialogFooter from "@/shared/components/DialogFooter.vue";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{ git: GitApi }>();
const visible = defineModel<boolean>("visible", { default: false });

const mergeBranchSel = ref("");
const mergeSquash = ref(true);
const mergeMessage = ref("");

const mergeableBranches = computed(() =>
  props.git.branches.value
    .filter((b) => !b.is_current && !b.name.endsWith("/HEAD"))
    .map((b) => ({ label: b.is_remote ? `${b.name} (remote)` : b.name, value: b.name })),
);

watch(visible, (v) => {
  if (v) {
    mergeBranchSel.value = "";
    mergeSquash.value = true;
    mergeMessage.value = "";
  }
});

async function doMerge() {
  if (!mergeBranchSel.value) return;
  const ok = await props.git.mergeBranch(mergeBranchSel.value, mergeSquash.value, mergeMessage.value);
  if (ok) visible.value = false;
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Merge branch" :style="{ width: '480px' }">
    <div class="flex flex-col gap-3">
      <p class="text-sm text-secondary">
        Merge một branch vào <strong class="text-ink">{{ git.info.value?.current_branch }}</strong>:
      </p>
      <Select
        v-model="mergeBranchSel"
        :options="mergeableBranches"
        option-label="label"
        option-value="value"
        placeholder="Chọn branch nguồn…"
        filter
        class="w-full"
      />
      <div class="flex items-center gap-2">
        <Checkbox v-model="mergeSquash" binary input-id="merge-squash" />
        <label for="merge-squash" class="text-sm text-ink">Squash &amp; merge (gộp thành 1 commit)</label>
      </div>
      <div v-if="mergeSquash">
        <label class="mb-1 block text-xs font-bold text-muted">Commit message (tùy chọn)</label>
        <InputText
          v-model="mergeMessage"
          :placeholder="`Squash merge branch '${mergeBranchSel || '...'}'`"
          class="w-full"
        />
      </div>
      <p class="text-xs text-muted">
        Nếu có xung đột, merge sẽ tạm dừng — giải quyết ở tab Changes rồi commit để hoàn tất.
      </p>
    </div>
    <template #footer>
      <DialogFooter
        cancel-label="Hủy"
        :confirm-label="mergeSquash ? 'Squash & merge' : 'Merge'"
        confirm-icon="pi pi-code-branch"
        :confirm-disabled="!mergeBranchSel || !!git.busyMessage.value"
        @cancel="visible = false"
        @confirm="doMerge"
      />
    </template>
  </Dialog>
</template>
