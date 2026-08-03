import { type Theme } from '@/bindings/bindings';

export const applyTheme = (theme: Theme) => {
  const root = document.documentElement;
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  switch (theme) {
    case 'Dark':
      root.classList.add('dark');
      break;

    case 'Light':
      root.classList.remove('dark');
      break;

    case 'System':
      root.classList.toggle('dark', mediaQuery.matches);
      break;
  }
};
