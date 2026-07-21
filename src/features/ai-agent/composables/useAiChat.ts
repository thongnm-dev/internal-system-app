import { computed, ref, watch } from "vue";
import { aiChatComplete } from "@/tauri/commands/ai-chat";
import { canUseTauriRuntime, friendlyError } from "@/tauri/commands/_base";
import { useToast } from "@/shared/composables/useToast";
import type { AiChatMessage } from "@/_/types/ai-chat";

export type ProviderId = "gemini" | "groq";

export type AiProvider = {
  id: ProviderId;
  label: string;
  icon: string;
  models: string[];
};

export type ChatRole = "user" | "assistant";

export type ChatMessage = {
  id: string;
  role: ChatRole;
  content: string;
  provider: ProviderId;
  model: string;
  createdAt: number;
};

export type ChatSession = {
  id: string;
  title: string;
  messages: ChatMessage[];
  providerId: ProviderId;
  model: string;
  createdAt: number;
  updatedAt: number;
};

export const AI_PROVIDERS: AiProvider[] = [
  {
    id: "gemini",
    label: "Gemini",
    icon: "pi pi-google",
    models: ["gemini-3.1-flash-lite","gemini-3.5-flash"],
  },
  {
    id: "groq",
    label: "Groq",
    icon: "pi pi-bolt",
    models: ["openai/gpt-oss-120b", "openai/gpt-oss-20b"],
  },
];

const SESSIONS_KEY = "msh.ai.chat.sessions";
const DEFAULT_PROVIDER: ProviderId = "gemini";

function providerFor(id: ProviderId): AiProvider {
  return AI_PROVIDERS.find((p) => p.id === id) ?? AI_PROVIDERS[0];
}

function newId() {
  return typeof crypto !== "undefined" && "randomUUID" in crypto
    ? crypto.randomUUID()
    : `id-${Date.now()}-${Math.round(Math.random() * 1e6)}`;
}

function loadSessions(): ChatSession[] {
  try {
    const saved = window.localStorage.getItem(SESSIONS_KEY);
    if (!saved) return [];
    const parsed = JSON.parse(saved) as ChatSession[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

export function useAiChat() {
  const toast = useToast();
  const providers = AI_PROVIDERS;
  const sessions = ref<ChatSession[]>(loadSessions());
  const activeId = ref<string | null>(sessions.value[0]?.id ?? null);
  const draft = ref("");
  const sending = ref(false);

  const activeSession = computed(() => sessions.value.find((s) => s.id === activeId.value) ?? null);
  const providerId = computed<ProviderId>(() => activeSession.value?.providerId ?? DEFAULT_PROVIDER);
  const model = computed<string>(() => activeSession.value?.model ?? providerFor(DEFAULT_PROVIDER).models[0]);
  const messages = computed<ChatMessage[]>(() => activeSession.value?.messages ?? []);
  const availableModels = computed(() => providerFor(providerId.value).models);
  const canSend = computed(() => draft.value.trim().length > 0 && !sending.value);

  watch(
    sessions,
    (value) => {
      try {
        window.localStorage.setItem(SESSIONS_KEY, JSON.stringify(value));
      } catch {
        /* ignore quota / serialization errors */
      }
    },
    { deep: true },
  );

  function createSession(): ChatSession {
    const now = Date.now();
    const session: ChatSession = {
      id: newId(),
      title: "New chat",
      messages: [],
      providerId: DEFAULT_PROVIDER,
      model: providerFor(DEFAULT_PROVIDER).models[0],
      createdAt: now,
      updatedAt: now,
    };
    sessions.value = [session, ...sessions.value];
    activeId.value = session.id;
    // Return the reactive proxy (not the raw literal) so callers mutate the
    // tracked instance — otherwise pushes bypass the deep watch and never persist.
    return activeSession.value ?? session;
  }

  function openSession(id: string) {
    activeId.value = id;
    draft.value = "";
  }

  function deleteSession(id: string) {
    sessions.value = sessions.value.filter((s) => s.id !== id);
    if (activeId.value === id) activeId.value = sessions.value[0]?.id ?? null;
  }

  function ensureActive(): ChatSession {
    return activeSession.value ?? createSession();
  }

  function selectProvider(id: ProviderId) {
    const session = ensureActive();
    if (id === session.providerId) return;
    session.providerId = id;
    // Reset to the first model of the newly selected provider.
    session.model = providerFor(id).models[0];
  }

  function selectModel(value: string) {
    ensureActive().model = value;
  }

  async function send() {
    const text = draft.value.trim();
    if (!text || sending.value) return;

    const session = ensureActive();
    const now = Date.now();
    session.messages.push({
      id: newId(),
      role: "user",
      content: text,
      provider: session.providerId,
      model: session.model,
      createdAt: now,
    });
    if (session.messages.filter((m) => m.role === "user").length === 1) {
      session.title = text.slice(0, 48) + (text.length > 48 ? "…" : "");
    }
    session.updatedAt = now;
    draft.value = "";
    sending.value = true;

    try {
      const reply = canUseTauriRuntime()
        ? await requestCompletion(session)
        : await placeholderReply(text);
      session.messages.push({
        id: newId(),
        role: "assistant",
        content: reply,
        provider: session.providerId,
        model: session.model,
        createdAt: Date.now(),
      });
      session.updatedAt = Date.now();
    } catch (e) {
      toast.error(friendlyError(e));
    } finally {
      sending.value = false;
    }
  }

  /** Gửi toàn bộ lịch sử hội thoại của session lên backend. */
  async function requestCompletion(session: ChatSession): Promise<string> {
    const history: AiChatMessage[] = session.messages.map((m) => ({
      role: m.role,
      content: m.content,
    }));
    const result = await aiChatComplete({
      provider: session.providerId,
      model: session.model,
      messages: history,
    });
    return result.content;
  }

  return {
    providers,
    sessions,
    activeId,
    activeSession,
    providerId,
    model,
    messages,
    draft,
    sending,
    availableModels,
    canSend,
    createSession,
    openSession,
    deleteSession,
    selectProvider,
    selectModel,
    send,
  };
}

function placeholderReply(prompt: string): Promise<string> {
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve(`Backend chưa được kết nối. Đây là phản hồi mẫu cho: "${prompt}".`);
    }, 500);
  });
}
