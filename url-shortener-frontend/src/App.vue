<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';

import { createShortLink, LinkApiError, type ShortLink } from './services/links';
import { registerShortLinkTool } from './services/webmcp';

const destinationUrl = ref('');
const isSubmitting = ref(false);
const invalidDestination = ref(false);
const shortLink = ref<ShortLink | null>(null);
const errorMessage = ref('');
const copyMessage = ref('');

const shortUrl = computed(() => {
  if (!shortLink.value) return '';
  return new URL(shortLink.value.short_path, window.location.origin).href;
});

async function createAndShowShortLink(destination: string, signal?: AbortSignal): Promise<ShortLink> {
  if (isSubmitting.value) {
    throw new LinkApiError('A link request is already in progress. Wait for it to finish, then retry.', 'request_in_progress');
  }

  isSubmitting.value = true;
  invalidDestination.value = false;
  errorMessage.value = '';
  copyMessage.value = '';
  shortLink.value = null;

  try {
    const createdLink = await createShortLink(destination, signal);
    destinationUrl.value = destination;
    shortLink.value = createdLink;
    return createdLink;
  } catch (error) {
    if (error instanceof LinkApiError) {
      invalidDestination.value = error.code === 'invalid_destination_url';
      errorMessage.value = error.message;
      throw error;
    } else {
      errorMessage.value = 'The short link could not be created. Try again later.';
      throw new Error(errorMessage.value);
    }
  } finally {
    isSubmitting.value = false;
  }
}

async function submitLink() {
  try {
    await createAndShowShortLink(destinationUrl.value);
  } catch {
    // The request helper has already set the visible validation or service error.
  }
}

async function copyShortLink() {
  try {
    await navigator.clipboard.writeText(shortUrl.value);
    copyMessage.value = 'Short link copied.';
  } catch {
    copyMessage.value = 'Copy is unavailable. Select the short link and copy it.';
  }
}

let unregisterShortLinkTool: (() => void) | undefined;

onMounted(() => {
  const registration = registerShortLinkTool(
    document.modelContext,
    (destination, signal) => createAndShowShortLink(destination, signal),
    window.location.origin,
  );
  unregisterShortLinkTool = registration?.unregister;
});

onBeforeUnmount(() => unregisterShortLinkTool?.());
</script>

<template>
  <main class="page-shell">
    <header class="page-heading">
      <p class="eyebrow">
        SHORT FORM
      </p>
      <h1>Make a long link easier to share.</h1>
      <p class="intro">
        Create one simple short link for any page you want to share.
      </p>
    </header>

    <section
      class="form-card"
      aria-labelledby="form-title"
    >
      <div class="card-heading">
        <h2 id="form-title">
          Create a short link
        </h2>
        <p>Paste the full HTTP or HTTPS address below.</p>
      </div>

      <form
        class="link-form"
        @submit.prevent="submitLink"
      >
        <div class="field">
          <label for="destination-url">Destination URL</label>
          <input
            id="destination-url"
            v-model="destinationUrl"
            type="url"
            inputmode="url"
            placeholder="https://example.com/my-page"
            autocomplete="url"
            maxlength="2048"
            required
            :aria-invalid="invalidDestination"
            :aria-describedby="invalidDestination ? 'url-help url-error' : 'url-help'"
          >
          <p
            id="url-help"
            class="field-help"
          >
            Maximum 2 KiB. The service does not open or scan the destination.
          </p>
          <p
            v-if="invalidDestination"
            id="url-error"
            class="field-error"
            role="alert"
          >
            {{ errorMessage }}
          </p>
        </div>

        <button
          class="button button-primary"
          type="submit"
          :disabled="isSubmitting"
        >
          <span
            v-if="isSubmitting"
            class="spinner"
            aria-hidden="true"
          />
          {{ isSubmitting ? 'Creating link…' : 'Create short link' }}
        </button>
      </form>

      <section
        v-if="shortLink"
        class="result-panel"
        aria-labelledby="result-title"
      >
        <div class="result-heading">
          <h2 id="result-title">
            Your short link is ready
          </h2>
          <p>This link redirects to the destination you submitted.</p>
        </div>
        <div class="result-row">
          <a
            class="short-link"
            :href="shortUrl"
            data-testid="short-link"
          >{{ shortUrl }}</a>
          <button
            class="button button-secondary copy-button"
            type="button"
            @click="copyShortLink"
          >
            Copy link
          </button>
        </div>
        <output
          class="status-message"
          aria-live="polite"
        >
          {{ copyMessage }}
        </output>
      </section>

      <p
        v-if="errorMessage && !invalidDestination"
        class="service-error"
        role="alert"
      >
        {{ errorMessage }}
      </p>
    </section>

    <p class="privacy-note">
      Short links are public. Anyone who has a link can follow its destination.
    </p>
  </main>
</template>
