import './styles/app.css';
import TrayPopover from './lib/components/tray/TrayPopover.svelte';
import { mount } from 'svelte';
// Theme and density follow the main window (shared localStorage).
import './lib/stores/theme';
import './lib/stores/density';

const app = mount(TrayPopover, {
  target: document.getElementById('app')!,
});

export default app;
