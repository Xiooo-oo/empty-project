<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';

  type Action = 'idle' | 'walk' | 'sleep' | 'blink' | 'jump' | 'meow';
  const walkFrames = ['/cats/walk-1.png', '/cats/walk-2.png', '/cats/walk-3.png'];
  let action: Action = 'idle';
  let frame = 0;
  let menuOpen = false;
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
    void callNative('set_interactive', { interactive: false });
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
    frameTimer = setInterval(() => {
      if (action === 'walk') frame = (frame + 1) % walkFrames.length;
      if (!desktopRuntime && !resting && Date.now() - lastActivity > 180_000 && action !== 'sleep') setAction('sleep');
    }, 500);

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

<svelte:head><title>咪咪桌宠</title></svelte:head>

<main class="pet-window" oncontextmenu={onContextMenu}>
  {#if bubble}<div class="speech" aria-live="polite">{bubble}</div>{/if}
  <div class="menu-anchor">
    {#if menuOpen}
      <nav class="pet-menu" aria-label="猫咪动作">
        <button onclick={() => runAction('blink')}>眨眨眼</button>
        <button onclick={() => runAction('jump')}>跳一下</button>
        <button onclick={() => runAction('meow')}>喵一声</button>
        <button onclick={() => runAction('walk')}>走两步</button>
        <span class="menu-divider"></span>
        <button onclick={() => void toggleRest()}>{resting ? '结束休息' : '开始休息'}</button>
        <button onclick={() => { closeMenu(); void callNative('quit_app'); }}>退出桌宠</button>
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
    alt="黑白奶牛猫 Mimi"
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
