<script lang="ts">
  import { t } from '$lib/i18n';

  let {
    onClose,
  }: {
    onClose: () => void;
  } = $props();

  function handleOverlayClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

<div
  class="modal-overlay"
  onclick={handleOverlayClick}
  role="dialog"
  aria-label="Keyboard shortcuts"
>
  <div class="modal-content shortcuts-modal">
    <div class="modal-header">
      <h2 class="modal-title">{$t('shortcuts.title')}</h2>
      <button class="btn btn-ghost btn-icon" onclick={onClose}>
        <svg
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <div class="shortcuts-grid">
      <section class="shortcut-section">
        <h3 class="section-label">{$t('shortcuts.section.categorize')}</h3>
        <div class="shortcut-row">
          <kbd>1</kbd>
          <span>{$t('shortcuts.trash')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>2</kbd>
          <span>{$t('shortcuts.best')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>3</kbd>
          <span>{$t('shortcuts.draft')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>4</kbd>
          <span>{$t('shortcuts.review')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>Delete</kbd>
          <span>{$t('shortcuts.move_trash')}</span>
        </div>
      </section>

      <section class="shortcut-section">
        <h3 class="section-label">{$t('shortcuts.section.navigate')}</h3>
        <div class="shortcut-row">
          <kbd>←</kbd> <kbd>→</kbd>
          <span>{$t('shortcuts.prev_next')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>↑</kbd> <kbd>↓</kbd>
          <span>{$t('shortcuts.prev_next')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>Space</kbd>
          <span>{$t('shortcuts.toggle_view')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>Esc</kbd>
          <span>{$t('shortcuts.back_grid')}</span>
        </div>
      </section>

      <section class="shortcut-section">
        <h3 class="section-label">{$t('shortcuts.section.actions')}</h3>
        <div class="shortcut-row">
          <kbd>Ctrl</kbd>+<kbd>Z</kbd>
          <span>{$t('shortcuts.undo')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>I</kbd>
          <span>{$t('shortcuts.toggle_info')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>?</kbd>
          <span>{$t('shortcuts.toggle_shortcuts')}</span>
        </div>
      </section>

      <section class="shortcut-section">
        <h3 class="section-label">{$t('shortcuts.section.ratings')}</h3>
        <div class="shortcut-row">
          <kbd>0</kbd> or <kbd>5</kbd>
          <span>{$t('shortcuts.star_rating')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>6</kbd> – <kbd>9</kbd>
          <span>{$t('shortcuts.color_label')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>A</kbd>
          <span>{$t('shortcuts.toggle_autoadvance')}</span>
        </div>
      </section>

      <section class="shortcut-section">
        <h3 class="section-label">{$t('shortcuts.section.diagnostics')}</h3>
        <div class="shortcut-row">
          <kbd>C</kbd>
          <span>{$t('shortcuts.cycle_compare')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>Z</kbd>
          <span>{$t('shortcuts.sync_zoom')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>H</kbd>
          <span>{$t('shortcuts.toggle_histogram')}</span>
        </div>
        <div class="shortcut-row">
          <kbd>O</kbd>
          <span>{$t('shortcuts.cycle_overlays')}</span>
        </div>
      </section>
    </div>

    <div class="modal-footer">
      <p class="footer-tip">
        {@html $t('shortcuts.footer_tip', {keys: '<kbd>1</kbd>–<kbd>4</kbd>'})}
      </p>
    </div>
  </div>
</div>

<style>
  .shortcuts-modal {
    max-width: 600px;
    padding: var(--space-6);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-6);
  }

  .modal-title {
    font-size: var(--text-lg);
    font-weight: 700;
    color: var(--text-primary);
  }

  .shortcuts-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-6);
  }

  .shortcut-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .section-label {
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.8px;
    margin-bottom: var(--space-1);
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    color: var(--text-secondary);
  }

  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    padding: 0 6px;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 600;
    color: var(--text-secondary);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-sm);
    box-shadow: 0 1px 0 var(--border-subtle);
  }

  .modal-footer {
    margin-top: var(--space-6);
    padding-top: var(--space-4);
    border-top: 1px solid var(--border-subtle);
  }

  .footer-tip {
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    text-align: center;
  }

  .footer-tip kbd {
    height: 18px;
    min-width: 18px;
    font-size: 9px;
    padding: 0 4px;
  }
</style>
