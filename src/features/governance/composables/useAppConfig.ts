import { ref } from "vue";
import { canUseTauriRuntime } from "@/tauri/commands/_base";
import { useGlobalLoading } from "@/shared/composables/useGlobalLoading";
import { getAppConfig, saveAppConfig } from "@/tauri/commands/app-config";
import type { ConfigSection, ConfigEntry } from "@/_/types/app-config";

export function useAppConfig() {
  const globalLoading = useGlobalLoading();
  const sections = ref<ConfigSection[]>([]);
  const configPath = ref("");
  const loading = ref(false);
  const error = ref("");
  const isDirty = ref(false);

  async function load() {
    if (!canUseTauriRuntime()) return;
    loading.value = true;
    error.value = "";
    try {
      const data = await getAppConfig();
      sections.value = data.sections;
      configPath.value = data.config_path;
      isDirty.value = false;
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  function updateEntry(sectionIndex: number, entryIndex: number, value: string) {
    sections.value[sectionIndex].entries[entryIndex].value = value;
    isDirty.value = true;
  }

  function addEntry(sectionIndex: number) {
    sections.value[sectionIndex].entries.push({ key: "", value: "" });
    isDirty.value = true;
  }

  function updateEntryKey(sectionIndex: number, entryIndex: number, key: string) {
    sections.value[sectionIndex].entries[entryIndex].key = key;
    isDirty.value = true;
  }

  function removeEntry(sectionIndex: number, entryIndex: number) {
    sections.value[sectionIndex].entries.splice(entryIndex, 1);
    isDirty.value = true;
  }

  function addSection() {
    sections.value.push({ name: "", entries: [{ key: "", value: "" }] });
    isDirty.value = true;
  }

  function updateSectionName(sectionIndex: number, name: string) {
    sections.value[sectionIndex].name = name;
    isDirty.value = true;
  }

  function removeSection(sectionIndex: number) {
    sections.value.splice(sectionIndex, 1);
    isDirty.value = true;
  }

  async function save(): Promise<boolean> {
    const invalid = sections.value.find((s) => !s.name.trim());
    if (invalid) {
      error.value = "Section name cannot be empty.";
      return false;
    }

    for (const section of sections.value) {
      const emptyKey = section.entries.find((e) => !e.key.trim());
      if (emptyKey) {
        error.value = `Key cannot be empty in section [${section.name}].`;
        return false;
      }
    }

    error.value = "";
    globalLoading.start();
    try {
      const data = await saveAppConfig({ sections: sections.value });
      sections.value = data.sections;
      configPath.value = data.config_path;
      isDirty.value = false;
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      globalLoading.stop();
    }
  }

  function discard() {
    globalLoading.run(() => load());
  }

  function init() {
    globalLoading.run(() => load());
  }

  return {
    sections,
    configPath,
    loading,
    error,
    isDirty,
    updateEntry,
    updateEntryKey,
    addEntry,
    removeEntry,
    addSection,
    updateSectionName,
    removeSection,
    save,
    discard,
    init,
  };
}
