// jsdom lacks a few browser APIs Svelte transitions and DataTable touch.
if (!Element.prototype.scrollIntoView) {
  Element.prototype.scrollIntoView = function () {};
}

if (!Element.prototype.animate) {
  // Svelte 5 transitions use the Web Animations API. Finish immediately.
  Element.prototype.animate = function () {
    const anim = {
      onfinish: null as null | (() => void),
      oncancel: null as null | (() => void),
      cancel() {},
      finish() {},
      play() {},
      pause() {},
      reverse() {},
      currentTime: 0,
      playState: 'finished',
      finished: Promise.resolve(),
      addEventListener() {},
      removeEventListener() {},
    };
    queueMicrotask(() => anim.onfinish?.());
    return anim as unknown as Animation;
  };
}

if (!window.matchMedia) {
  window.matchMedia = ((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener() {},
    removeEventListener() {},
    addListener() {},
    removeListener() {},
    dispatchEvent: () => false,
  })) as unknown as typeof window.matchMedia;
}
