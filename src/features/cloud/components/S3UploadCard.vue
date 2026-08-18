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

// `!`-prefixed (Tailwind important) because PrimeVue's own .p-tree-node-icon
// color rule is injected at runtime and otherwise wins the cascade.
function fileIconClass(name: string): string {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  if (["xlsx", "xls", "xlsm", "csv"].includes(ext)) return "pi pi-file-excel !text-green-600";
  if (ext === "txt") return "pi pi-file-edit !text-muted";
  return "pi pi-file !text-muted";
}

function buildSubfolderNodes(
  entries: { file: ScannedFile; segments: string[] }[],
  parentKey: string,
): TreeNode[] {
  const folderMap = new Map<string, { file: ScannedFile; segments: string[] }[]>();
  const fileNodes: TreeNode[] = [];

  for (const entry of entries) {
    if (entry.segments.length === 0) {
      fileNodes.push({
        key: `file-${entry.file.filePath}`,
        label: entry.file.name,
        icon: fileIconClass(entry.file.name),
        data: entry.file,
      });
      continue;
    }
    const [head, ...rest] = entry.segments;
    if (!folderMap.has(head)) folderMap.set(head, []);
    folderMap.get(head)!.push({ file: entry.file, segments: rest });
  }

  const folderNodes: TreeNode[] = Array.from(folderMap.entries()).map(([name, subEntries]) => {
    const folderKey = `${parentKey}/${name}`;
    return {
      key: `subfolder-${folderKey}`,
      label: name,
      icon: "pi pi-folder !text-orange-500",
      children: buildSubfolderNodes(subEntries, folderKey),
    };
  });

  return [...folderNodes, ...fileNodes];
}

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

  const children: TreeNode[] = Object.entries(grouped).map(([folder, files]) => {
    const folderKey = `folder-${folder}`;
    const entries = files.map((f) => ({
      file: f,
      segments: f.subPath ? f.subPath.split("/") : [],
    }));
    return {
      key: folderKey,
      label: folder,
      icon: "pi pi-folder !text-orange-500",
      children: buildSubfolderNodes(entries, folderKey),
    };
  });

  return [
    {
      key: "root",
      label: "Danh sách thư mục đã chọn",
      icon: "pi pi-folder-open !text-orange-500",
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
  <div class="grid grid-cols-1 rounded bg-panel shadow">
    <!-- Header -->
    <div class="flex flex-col border-b border-divider px-4 py-2">
      <div class="flex items-center justify-between">
        <div class="flex flex-1 cursor-pointer items-center gap-2" @click="toggle">
          <i :class="['pi text-lg text-amber-500', expanded ? 'pi-folder-open' : 'pi-folder']" />
          <span class="text-lg font-bold text-ink">
            {{ awsStorage.nameAlias || awsStorage.name }}
            <span class="text-danger">({{ folderCount }})</span>
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
        class="flex flex-col items-center justify-center py-8 text-muted"
      >
        <i class="pi pi-inbox mb-2 text-3xl" />
        <span class="text-sm">Chưa chọn tập tin nào</span>
      </div>
    </div>
  </div>
</template>
