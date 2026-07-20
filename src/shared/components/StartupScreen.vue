<script setup lang="ts">
const companyBackground =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAARGVYSWZNTQAqAAAACAABh2kABAAAAAEAAAAaAAAAAAADoAEAAwAAAAEAAQAAoAIABAAAAAEAAABAoAMABAAAAAEAAABAAAAAAEZRQrAAAAFmSURBVHgB7VrtCoMwDGzH/u+R9M2sb1afzC0yIUhtQW0SyAlj/XK5XO7a/lgMKX2C4+flOPctdRAABThnABZwLoAABUABzhmABZwLAJsgLAALOGcAFnAuAJwCsAAs4JwBWMC5AHAKwALeLfB+koB1mk5/blmWMOZcnM/jGIZh2OZq64ov3xwUswAlSASlm4Cffr0bAfM8h/j/cNDjr9qWnm4E8CQj65ASEutrN0UI0E6yFl+cANrkUg2R8JwIAStLKp+cBGyJaPPRY5Ajn347/vFQtFZ9wtuNAE4GdelUSMdBA/1uBFxNWNoiIntAq9D7LbC1rse8CgF09U17NpXr876k53c3C7RAlzbJq7ZpxarNqyigBEgjecIR8UfJUjkcjZmxgBbnIECLeStxoQArldDCAQVoMW8lLhRgpRJaOKAALeatxIUCrFRCCwcUoMW8lbhQgJVKaOKAArSYtxIXCrBSCS0cX3CdMewlzOB4AAAAAElFTkSuQmCC";
</script>

<template>
  <main
    class="force-light flex h-screen min-h-[640px] min-w-[900px] items-center justify-center overflow-hidden bg-canvas text-ink"
    data-theme="light"
  >
    <section
      aria-live="polite"
      aria-label="Trang thai khoi dong he thong"
      class="flex select-none flex-col items-center justify-center gap-7 px-8 text-center"
      role="status"
    >
      <img
        alt="PJ Yuji"
        class="h-44 w-44 rounded-md object-contain shadow-[0_20px_70px_rgba(15,23,42,0.16)]"
        :draggable="false"
        :src="companyBackground"
      />

      <div class="flex flex-col items-center gap-4">
        <span aria-hidden="true" class="startup-loading-gif">
          <span />
          <span />
          <span />
          <span />
        </span>
        <p class="text-base font-semibold text-secondary">
          Đang khởi động hệ thống. Vui lòng chờ!
        </p>
      </div>
    </section>
  </main>
</template>
