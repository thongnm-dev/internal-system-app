<script setup lang="ts">
import { ref, computed, watchEffect } from "vue";
import Button from "primevue/button";
import Tree from "primevue/tree";
import type { TreeNode } from "primevue/treenode";
import type { AwsStorage, ScannedFile } from "@/_/types/s3";

const props = defineProps<{
  awsStorage: AwsStorage;
  uploadedId?: string;
}>();

const emit = defineEmits<{
  upload: [params: { aws_storage: AwsStorage; is_folder_same_name: boolean; selected_items: ScannedFile[] }];
  clear: [];
  scanFolder: [callback: (files: ScannedFile[]) => void];
}>();

const expanded = ref(true);
const items = ref<ScannedFile[]>([]);
const selectedKeys = ref<Record<string, { checked: boolean; partialChecked: boolean }>>({});
const expandedKeys = ref<Record<string, boolean>>({});

const isUploadable = computed(() => {
  if (props.uploadedId && props.uploadedId === props.awsStorage.code) return false;
  return true;
});

const treeNodes = computed<TreeNode[]>(() => {
  if (items.value.length === 0) return [];

  const grouped = items.value.reduce(
    (acc, item) => {
      if (!acc[item.parentName]) acc[item.parentName] = [];
      acc[item.parentName].push(item);
      return acc;
    },
    {} as Record<string, ScannedFile[]>,
  );

  const children: TreeNode[] = Object.entries(grouped).map(([folder, files]) => ({
    key: `folder-${folder}`,
    label: folder,
    icon: "pi pi-folder",
    children: files.map((f) => ({
      key: `file-${f.filePath}`,
      label: f.name,
      icon: "pi pi-file",
      data: f,
    })),
  }));

  return [
    {
      key: "root",
      label: "Danh sách thư mục đã chọn",
      icon: "pi pi-folder-open",
      children,
    },
  ];
});

const folderCount = computed(() => {
  if (items.value.length === 0) return 0;
  return new Set(items.value.map((f) => f.parentName)).size;
});

const selectedItems = computed<ScannedFile[]>(() => {
  const keys = Object.keys(selectedKeys.value).filter(
    (k) => k.startsWith("file-") && selectedKeys.value[k]?.checked,
  );
  return items.value.filter((f) => keys.includes(`file-${f.filePath}`));
});

watchEffect(() => {
  if (treeNodes.value.length > 0) {
    const keys: Record<string, boolean> = {};
    const sel: Record<string, { checked: boolean; partialChecked: boolean }> = {};
    function walk(nodes: TreeNode[]) {
      for (const n of nodes) {
        keys[n.key as string] = true;
        sel[n.key as string] = { checked: true, partialChecked: false };
        if (n.children) walk(n.children);
      }
    }
    walk(treeNodes.value);
    expandedKeys.value = keys;
    selectedKeys.value = sel;
  }
});

function toggle() {
  expanded.value = !expanded.value;
}

function addAttachment() {
  emit("scanFolder", (files: ScannedFile[]) => {
    if (files.length > 0) {
      const existingPaths = new Set(items.value.map((f) => f.filePath));
      const newFiles = files.filter((f) => !existingPaths.has(f.filePath));
      items.value = [...items.value, ...newFiles];
    }
  });
}

function clearItems() {
  items.value = [];
  selectedKeys.value = {};
  emit("clear");
}

function handleUpload() {
  if (selectedItems.value.length === 0) return;
  emit("upload", {
    aws_storage: props.awsStorage,
    is_folder_same_name: props.awsStorage.code === "011",
    selected_items: selectedItems.value,
  });
}
</script>

<template>
  <div class="grid grid-cols-1 rounded bg-surface-0 shadow dark:bg-surface-900">
    <!-- Header -->
    <div class="flex flex-col border-b border-surface-200 px-4 py-2 dark:border-surface-700">
      <div class="flex items-center justify-between">
        <div class="flex flex-1 cursor-pointer items-center gap-2" @click="toggle">
          <i :class="['pi text-lg text-orange-500', expanded ? 'pi-folder-open' : 'pi-folder']" />
          <span class="text-lg font-bold text-surface-800 dark:text-surface-100">
            {{ awsStorage.nameAlias || awsStorage.name }}
            <span class="text-red-600">({{ folderCount }})</span>
          </span>
        </div>
        <div class="flex items-center gap-2">
          <Button
            v-if="items.length > 0"
            label="Dọn sạch"
            icon="pi pi-eraser"
            severity="danger"
            outlined
            size="small"
            @click="clearItems"
          />
          <Button
            label="Chọn tập tin"
            icon="pi pi-folder-plus"
            severity="secondary"
            outlined
            size="small"
            @click="addAttachment"
          />
          <Button
            v-if="items.length > 0 && selectedItems.length > 0"
            label="Tải lên"
            icon="pi pi-upload"
            size="small"
            :disabled="selectedItems.length === 0 || !isUploadable"
            @click="handleUpload"
          />
        </div>
      </div>
    </div>

    <!-- Tree view -->
    <div v-if="expanded" class="max-h-[280px] overflow-y-auto py-2 px-4">
      <Tree
        v-if="treeNodes.length > 0"
        v-model:selectionKeys="selectedKeys"
        v-model:expandedKeys="expandedKeys"
        :value="treeNodes"
        selection-mode="checkbox"
        class="w-full border-none"
      />
      <div
        v-else
        class="flex flex-col items-center justify-center py-8 text-surface-400"
      >
        <i class="pi pi-inbox mb-2 text-3xl" />
        <span class="text-sm">Chưa chọn tập tin nào</span>
      </div>
    </div>
  </div>
</template>
