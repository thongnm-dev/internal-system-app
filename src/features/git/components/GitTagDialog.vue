<script setup lang="ts">
import { ref, watch } from "vue";
import Button from "primevue/button";
import Checkbox from "primevue/checkbox";
import Dialog from "primevue/dialog";
import InputText from "primevue/inputtext";
import type { GitApi } from "../composables/useGit";

const props = defineProps<{
  git: GitApi;
  target: { hash: string; label: string };
}>();
const visible = defineModel<boolean>("visible", { default: false });

const tagName = ref("");
const tagMessage = ref("");
const tagAnnotated = ref(true);
const tagPush = ref(false);

watch(visible, (v) => {
  if (v) {
    tagName.value = "";
    tagMessage.value = "";
    tagAnnotated.value = true;
    tagPush.value = false;
    props.git.loadTags();
  }
});

async function doCreateTag() {
  const ok = await props.git.createTag(
    tagName.value,
    props.target.hash,
    tagMessage.value,
    tagAnnotated.value,
    tagPush.value,
  );
  if (ok) {
    tagName.value = "";
    tagMessage.value = "";
  }
}
</script>

<template>
  <Dialog v-model:visible="visible" modal header="Tạo tag" :style="{ width: '520px' }">
    <div class="flex flex-col gap-3">
      <p class="text-xs text-muted">Tạo tag tại: <strong class="text-ink">{{ target.label }}</strong></p>
      <div>
        <label class="mb-1 block text-sm font-medium text-ink">Tên tag</label>
        <InputText v-model="tagName" placeholder="v1.0.0" class="w-full" @keydown.enter="doCreateTag" />
      </div>
      <div class="flex items-center gap-2">
        <Checkbox v-model="tagAnnotated" binary input-id="tag-annotated" />
        <label for="tag-annotated" class="text-sm text-ink">Annotated (kèm message)</label>
      </div>
      <div v-if="tagAnnotated">
        <label class="mb-1 block text-sm font-medium text-ink">Message</label>
        <InputText v-model="tagMessage" placeholder="Mô tả cho tag" class="w-full" />
      </div>
      <div class="flex items-center gap-2">
        <Checkbox v-model="tagPush" binary input-id="tag-push" />
        <label for="tag-push" class="text-sm text-ink">Push tag lên origin sau khi tạo</label>
      </div>
      <div v-if="git.tags.value.length" class="mt-1">
        <p class="mb-1 text-xs font-bold uppercase tracking-wide text-muted">Tag hiện có</p>
        <div class="max-h-40 overflow-y-auto rounded-md border border-divider">
          <div
            v-for="t in git.tags.value"
            :key="t.name"
            class="flex items-center gap-2 border-b border-divider-light px-2.5 py-1.5 last:border-0"
          >
            <i class="pi pi-tag shrink-0 text-xs text-brand" />
            <span class="min-w-0 flex-1 truncate text-sm text-ink" :title="t.subject">{{ t.name }}</span>
            <span class="shrink-0 font-mono text-[11px] text-muted">{{ t.target }}</span>
            <button
              class="shrink-0 rounded p-1 text-muted transition-colors hover:text-red-600"
              title="Xóa tag (local)"
              @click="git.deleteTag(t.name, false)"
            >
              <i class="pi pi-trash text-xs" />
            </button>
          </div>
        </div>
      </div>
    </div>
    <template #footer>
      <Button size="small" outlined severity="secondary" @click="visible = false">Đóng</Button>
      <Button size="small" :disabled="!tagName.trim() || !!git.busyMessage.value" @click="doCreateTag">
        <i class="pi pi-tag mr-1.5" /> Tạo tag
      </Button>
    </template>
  </Dialog>
</template>
