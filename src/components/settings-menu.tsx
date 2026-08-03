import { Theme } from '@/bindings/bindings';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from './ui/dialog';
import { Field, FieldLabel } from './ui/field';
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select';
import { applyTheme } from '@/lib/theme';
import { useConfigStore } from '@/stores/config-stores';

export default function SettingsMenu({ openButton }: { openButton: React.ReactElement }) {
  const config = useConfigStore((state) => state.config);
  const setConfig = useConfigStore((state) => state.setConfig);
  const saveConfig = useConfigStore((state) => state.saveConfig);

  const selectedTheme = config?.settings?.theme ?? 'System';

  const OpenButton = openButton;

  const handleThemeChange = (value: Theme) => {
    if (!config) return;

    setConfig({
      ...config,
      settings: {
        ...(config.settings ?? {}),
        theme: value,
      },
    });

    applyTheme(value);
    void saveConfig();
  };

  return (
    <Dialog>
      <DialogTrigger tabIndex={-1} render={OpenButton} />
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
        </DialogHeader>
        <Field>
          <FieldLabel>App Theme</FieldLabel>
          <Select
            items={[
              { label: 'Light', value: 'Light' },
              { label: 'Dark', value: 'Dark' },
              { label: 'System', value: 'System' },
            ]}
            value={selectedTheme}
            onValueChange={(value) => handleThemeChange(value as Theme)}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select a theme" />
            </SelectTrigger>
            <SelectContent alignItemWithTrigger={false}>
              <SelectGroup>
                <SelectItem value="System">System</SelectItem>
                <SelectItem value="Dark">Dark</SelectItem>
                <SelectItem value="Light">Light</SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
        </Field>
      </DialogContent>
    </Dialog>
  );
}
