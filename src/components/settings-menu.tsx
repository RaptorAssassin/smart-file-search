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

export default function SettingsMenu({ openButton }: { openButton: React.ReactNode }) {
  return (
    <Dialog>
      <DialogTrigger tabIndex={-1}>{openButton}</DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
        </DialogHeader>
        <Field>
          <FieldLabel>Theme</FieldLabel>
          <Select
            items={[
              { label: 'Light', value: 'Light' },
              { label: 'Dark', value: 'Dark' },
              { label: 'System', value: 'System' },
            ]}
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
