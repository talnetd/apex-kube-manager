import './styles/app.css';
import DetailWindow from './lib/components/detail/DetailWindow.svelte';
import { mount } from 'svelte';

const app = mount(DetailWindow, {
  target: document.getElementById('app')!,
});

export default app;
