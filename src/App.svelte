<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';

  type Action = 'idle' | 'walk' | 'sleep' | 'blink' | 'jump' | 'meow';
  const walkFrames = [
    '/cats/walk-1-aligned.png',
    '/cats/walk-2-aligned.png',
    '/cats/walk-3-aligned.png',
    '/cats/walk-2-aligned.png',
  ];
  const walkFrameDuration = 125;
  const petSizes = [96, 128, 160] as const;
  let action: Action = 'idle';
  let frame = 0;
  let menuOpen = false;
  let menuPage: 'main' | 'actions' | 'sizes' = 'main';
  let petSize: number = 160;
  let resting = false;
  let bubble = '';
  let bubbleTimer: ReturnType<typeof setTimeout>;
  let frameTimer: ReturnType<typeof setInterval>;
  let actionTimer: ReturnType<typeof setTimeout>;
  let lastActivity = Date.now();
  const desktopRuntime = '__TAURI_INTERNALS__' in window;

  function callNative<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    if (!desktopRuntime) return Promise.resolve(undefined as T);
    return invoke<T>(command, args);
  }

  function listenNative<T>(event: string, handler: (event: { payload: T }) => void): Promise<UnlistenFn> {
    if (!desktopRuntime) return Promise.resolve(() => {});
    return listen<T>(event, handler);
  }

  $: image = action === 'sleep'
    ? '/cats/sleep.png'
    : action === 'walk'
      ? walkFrames[frame % walkFrames.length]
      : '/cats/idle.png';

  function setAction(next: Action, duration = 1400) {
    clearTimeout(actionTimer);
    action = next;
    if (next !== 'walk' && next !== 'sleep' && next !== 'idle') {
      actionTimer = setTimeout(() => action = resting ? 'sleep' : 'idle', duration);
    }
  }

  function noteActivity() {
    lastActivity = Date.now();
    if (action === 'sleep' && !resting) setAction('idle');
  }

  function closeMenu() {
    menuOpen = false;
    menuPage = 'main';
    void callNative('set_interactive', { interactive: false });
  }

  function changePetSize(size: number) {
    petSize = size;
    try { localStorage.setItem('xiaxia-deskpet-size', String(size)); } catch { /* Storage may be unavailable in private preview contexts. */ }
    if (desktopRuntime) void getCurrentWindow().setSize(new LogicalSize(size, size));
    closeMenu();
  }

  function showBubble(text: string) {
    bubble = text;
    clearTimeout(bubbleTimer);
    bubbleTimer = setTimeout(() => bubble = '', 1500);
  }

  async function meow() {
    showBubble('喵！');
    setAction('meow', 800);
    try {
      const ctx = new AudioContext();
      const oscillator = ctx.createOscillator();
      const gain = ctx.createGain();
      oscillator.type = 'sine';
      oscillator.frequency.setValueAtTime(530, ctx.currentTime);
      oscillator.frequency.exponentialRampToValueAtTime(790, ctx.currentTime + 0.13);
      oscillator.frequency.exponentialRampToValueAtTime(390, ctx.currentTime + 0.45);
      gain.gain.setValueAtTime(0.0001, ctx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.08, ctx.currentTime + 0.04);
      gain.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + 0.5);
      oscillator.connect(gain).connect(ctx.destination);
      oscillator.start();
      oscillator.stop(ctx.currentTime + 0.52);
      oscillator.onended = () => void ctx.close();
    } catch { /* Sound may be unavailable in a silent or locked audio session. */ }
  }

  function runAction(name: 'blink' | 'jump' | 'meow' | 'walk') {
    noteActivity();
    closeMenu();
    if (name === 'meow') void meow();
    else if (name === 'walk') {
      setAction('walk');
      actionTimer = setTimeout(() => setAction(resting ? 'sleep' : 'idle'), 3200);
    }
    else {
      setAction(name, name === 'blink' ? 520 : 780);
      if (name === 'jump') showBubble('喵～');
    }
  }

  function onContextMenu(event: MouseEvent) {
    event.preventDefault();
    noteActivity();
    menuOpen = !menuOpen;
    if (menuOpen) menuPage = 'main';
    void callNative('set_interactive', { interactive: menuOpen });
  }

  async function toggleRest() {
    resting = !resting;
    await callNative('set_resting', { resting });
    setAction(resting ? 'sleep' : 'idle');
    showBubble(resting ? '晚安…' : '我醒啦！');
    closeMenu();
  }

  onMount(() => {
    try {
      const savedSize = Number(localStorage.getItem('xiaxia-deskpet-size'));
      if (petSizes.includes(savedSize as typeof petSizes[number])) petSize = savedSize;
    } catch { /* Use the default size when storage is unavailable. */ }
    if (desktopRuntime) void getCurrentWindow().setSize(new LogicalSize(petSize, petSize));

    frameTimer = setInterval(() => {
      if (action === 'walk') frame = (frame + 1) % walkFrames.length;
      if (!desktopRuntime && !resting && Date.now() - lastActivity > 180_000 && action !== 'sleep') setAction('sleep');
    }, walkFrameDuration);

    const unlisten = listenNative<boolean>('resting-changed', ({ payload }) => {
      resting = payload;
      setAction(resting ? 'sleep' : 'idle');
    });
    const idleListener = listenNative<boolean>('system-idle', ({ payload }) => {
      if (payload && !resting) setAction('sleep');
      else if (!payload && !resting && action === 'sleep') setAction('idle');
    });
    void callNative<boolean>('system_idle').then((idle) => {
      if (idle && !resting) setAction('sleep');
    });
    const click = (event: PointerEvent) => {
      noteActivity();
      if (menuOpen && event.button !== 2 && !(event.target as Element).closest('.pet-menu')) closeMenu();
    };
    window.addEventListener('pointerdown', click);
    window.addEventListener('pointermove', noteActivity);
    window.addEventListener('keydown', noteActivity);
    return () => {
      clearInterval(frameTimer);
      clearTimeout(bubbleTimer);
      clearTimeout(actionTimer);
      window.removeEventListener('pointerdown', click);
      window.removeEventListener('pointermove', noteActivity);
      window.removeEventListener('keydown', noteActivity);
      void unlisten.then((off) => off());
      void idleListener.then((off) => off());
    };
  });
</script>

<svelte:head><title>虾虾桌宠</title></svelte:head>

<main class="pet-window" style={`--pet-size: ${petSize}px`} oncontextmenu={onContextMenu}>
  {#if bubble}<div class="speech" aria-live="polite">{bubble}</div>{/if}
  <div class="menu-anchor">
    {#if menuOpen}
      <nav class="pet-menu" aria-label="虾虾桌宠菜单">
        {#if menuPage === 'main'}
          <button onclick={() => menuPage = 'actions'}>动作…</button>
          <button onclick={() => menuPage = 'sizes'}>更改大小…</button>
          <span class="menu-divider"></span>
          <button onclick={() => void toggleRest()}>{resting ? '结束休息' : '开始休息'}</button>
          <button onclick={() => { closeMenu(); void callNative('quit_app'); }}>退出虾虾桌宠</button>
        {:else if menuPage === 'actions'}
          <button class="menu-back" onclick={() => menuPage = 'main'}>‹ 返回</button>
          <span class="menu-divider"></span>
          <div class="pet-action-options" aria-label="虾虾动作">
            <button onclick={() => runAction('blink')}>眨眨眼</button>
            <button onclick={() => runAction('jump')}>跳一下</button>
            <button onclick={() => runAction('meow')}>喵一声</button>
            <button onclick={() => runAction('walk')}>走两步</button>
          </div>
        {:else}
          <button class="menu-back" onclick={() => menuPage = 'main'}>‹ 返回</button>
          <span class="menu-divider"></span>
          <div class="size-label">桌宠大小（像素）</div>
          <div class="pet-size-options" aria-label="更改桌宠大小">
            {#each petSizes as size}
              <button class:selected={petSize === size} aria-pressed={petSize === size} onclick={() => changePetSize(size)}>{size}</button>
            {/each}
          </div>
        {/if}
      </nav>
    {/if}
  </div>
  <img
    class:blink={action === 'blink'}
    class:jump={action === 'jump'}
    class:meow={action === 'meow'}
    class:sleeping={action === 'sleep'}
    class:walking={action === 'walk'}
    src={image}
    alt="黑白奶牛猫虾虾"
    draggable="false"
    ondragstart={(event) => event.preventDefault()}
    onpointerdown={(event) => {
      if (event.button === 0 && !menuOpen) {
        noteActivity();
        void callNative('start_dragging');
      }
    }}
    onpointerenter={noteActivity}
  />
</main>
{#if !desktopRuntime}
  <aside class="preview-notice">浏览器预览模式：托盘、置顶和点击穿透需在 Windows 桌面版体验</aside>
{/if}
