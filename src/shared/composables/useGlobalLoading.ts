import { computed, ref } from "vue";

const stack = ref(0);

const isLoading = computed(() => stack.value > 0);

function start() {
  stack.value++;
}

function stop() {
  if (stack.value > 0) stack.value--;
}

async function run<T>(fn: () => Promise<T>): Promise<T> {
  start();
  try {
    return await fn();
  } finally {
    stop();
  }
}

export function useGlobalLoading() {
  return { isLoading, start, stop, run };
}
