import './styles/app.css';
import TerminalWindow from './lib/components/terminal/TerminalWindow.svelte';
import { mount } from 'svelte';

const app = mount(TerminalWindow, {
  target: document.getElementById('app')!,
});

export default app;
