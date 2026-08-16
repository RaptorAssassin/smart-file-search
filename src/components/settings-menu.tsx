import { AiConfig, AiProvider, Theme } from '@/bindings/bindings'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from './ui/dialog'
import {
  Field,
  FieldLabel,
  FieldDescription,
  FieldLegend,
  FieldSet,
  FieldContent,
} from './ui/field'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './ui/select'
import { Input } from './ui/input'
import { applyTheme } from '@/lib/theme'
import { useConfigStore } from '@/stores/config-store'
import { Checkbox } from './ui/checkbox'
import { useUIStore } from '@/stores/ui-store'
import { Button } from './ui/button'
import { SettingsIcon, RotateCcw } from 'lucide-react'
import KeyboardShortcut from './keyboard-shortcut'
import { SiGithub } from '@icons-pack/react-simple-icons'

const DEFAULT_OLLAMA_URL = 'http://localhost:11434'
const DEFAULT_OLLAMA_MODEL = 'gemma3:4b'
const EMBED_MODEL = 'nomic-embed-text'
const RECOMMENDED_MODELS = ['gemma3:4b']

export default function SettingsMenu() {
  const config = useConfigStore((state) => state.config)
  const setConfig = useConfigStore((state) => state.setConfig)
  const saveConfig = useConfigStore((state) => state.saveConfig)

  const selectedTheme = config?.settings?.theme ?? 'System'

  const ai = config?.settings?.ai ?? {}
  const provider = ai.provider ?? 'Ollama'
  const ollamaUrl = ai.ollama_url ?? DEFAULT_OLLAMA_URL
  const ollamaModel = ai.ollama_model ?? DEFAULT_OLLAMA_MODEL
  const customEndpoint = ai.custom_endpoint ?? ''
  const customApiKey = ai.custom_api_key ?? ''
  const customModel = ai.custom_model ?? ''
  const embeddingsEnabled = ai.embeddings_enabled ?? true
  const isCustomModel = ai.ollama_model_custom === true || !RECOMMENDED_MODELS.includes(ollamaModel)

  const handleThemeChange = (value: Theme) => {
    if (!config) return

    setConfig({
      ...config,
      settings: {
        ...(config.settings ?? {}),
        theme: value,
      },
    })

    applyTheme(value)
    void saveConfig()
  }

  const updateAi = (patch: Partial<AiConfig>) => {
    if (!config) return

    setConfig({
      ...config,
      settings: {
        ...(config.settings ?? {}),
        ai: {
          ...(config.settings?.ai ?? {}),
          ...patch,
        },
      },
    })

    void saveConfig()
  }

  const settingsOpen = useUIStore((state) => state.settingsOpen)
  const setSettingsOpen = useUIStore((state) => state.setSettingsOpen)

  return (
    <Dialog open={settingsOpen} onOpenChange={setSettingsOpen}>
      <DialogTrigger
        render={
          <Button type="button">
            <SettingsIcon />
            Settings
            <KeyboardShortcut identifier="settings" />
          </Button>
        }
      />
      <DialogContent className="sm:max-w-[28rem]">
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
        </DialogHeader>
        <div className="scrollbar-hidden max-h-[calc(100vh-12rem)] overflow-y-auto">
          {/* Models */}
          <FieldSet>
            <FieldLegend>Models</FieldLegend>
            <Field orientation="responsive">
              <FieldLabel>Provider</FieldLabel>
              <FieldContent>
                <Select
                  items={[
                    { label: 'Ollama', value: 'Ollama' },
                    { label: 'Custom', value: 'Custom' },
                  ]}
                  value={provider}
                  onValueChange={(value) => updateAi({ provider: value as AiProvider })}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select a provider" />
                  </SelectTrigger>
                  <SelectContent alignItemWithTrigger={false}>
                    <SelectGroup>
                      <SelectItem value="Ollama">Ollama</SelectItem>
                      <SelectItem value="Custom">Custom</SelectItem>
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </FieldContent>
            </Field>
            {provider === 'Ollama' && (
              <>
                <Field orientation="responsive">
                  <FieldLabel>Ollama URL</FieldLabel>
                  <FieldContent>
                    <div className="flex gap-2">
                      <Input
                        value={ollamaUrl}
                        onChange={(e) => updateAi({ ollama_url: e.target.value })}
                        placeholder={DEFAULT_OLLAMA_URL}
                      />
                      <Button
                        type="button"
                        variant="outline"
                        size="icon"
                        title="Reset to default"
                        aria-label="Reset Ollama URL"
                        onClick={() => updateAi({ ollama_url: DEFAULT_OLLAMA_URL })}
                      >
                        <RotateCcw />
                      </Button>
                    </div>
                  </FieldContent>
                </Field>
                <Field orientation="responsive">
                  <FieldLabel>Model</FieldLabel>
                  <FieldContent>
                    <Select
                      items={[
                        ...RECOMMENDED_MODELS.map((model) => ({ label: model, value: model })),
                        { label: 'Custom', value: 'custom' },
                      ]}
                      value={isCustomModel ? 'custom' : ollamaModel}
                      onValueChange={(value) => {
                        if (value === 'custom') {
                          updateAi({ ollama_model_custom: true })
                        } else {
                          updateAi({ ollama_model: value ?? '', ollama_model_custom: false })
                        }
                      }}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Select a model" />
                      </SelectTrigger>
                      <SelectContent alignItemWithTrigger={false}>
                        <SelectGroup>
                          {RECOMMENDED_MODELS.map((model) => (
                            <SelectItem key={model} value={model}>
                              {model}
                            </SelectItem>
                          ))}
                          <SelectItem value="custom">Custom</SelectItem>
                        </SelectGroup>
                      </SelectContent>
                    </Select>
                    {isCustomModel && (
                      <Input
                        value={ollamaModel}
                        onChange={(e) => updateAi({ ollama_model: e.target.value })}
                        placeholder={DEFAULT_OLLAMA_MODEL}
                      />
                    )}
                    {isCustomModel && (
                      <FieldDescription>
                        Enter any Ollama model installed locally. To install a new model, make sure
                        you got Ollama installed and run <code>ollama pull &lt;model-name&gt;</code>
                      </FieldDescription>
                    )}
                  </FieldContent>
                </Field>
              </>
            )}
            {provider === 'Custom' && (
              <>
                <Field orientation="responsive">
                  <FieldLabel>Endpoint</FieldLabel>
                  <FieldContent>
                    <Input
                      value={customEndpoint}
                      onChange={(e) => updateAi({ custom_endpoint: e.target.value })}
                      placeholder="https://api.openai.com/v1"
                    />
                    <FieldDescription>
                      Open-AI compatible endpoint. API usage might get high for users with many
                      files, so be careful and set limits.
                    </FieldDescription>
                  </FieldContent>
                </Field>
                <Field orientation="responsive">
                  <FieldLabel>API Key</FieldLabel>
                  <FieldContent>
                    <Input
                      type="password"
                      value={customApiKey}
                      onChange={(e) => updateAi({ custom_api_key: e.target.value })}
                      placeholder="sk-..."
                    />
                  </FieldContent>
                </Field>
                <Field orientation="responsive">
                  <FieldLabel>Model</FieldLabel>
                  <FieldContent>
                    <Input
                      value={customModel}
                      onChange={(e) => updateAi({ custom_model: e.target.value })}
                      placeholder="eg. gpt-4o-mini"
                    />
                  </FieldContent>
                </Field>
              </>
            )}
            <FieldSet>
              <FieldLegend variant="label">Embeddings</FieldLegend>
              <Field orientation="horizontal">
                <Checkbox
                  id="enable-embeddings"
                  name="enable-embeddings"
                  className="h-4 w-4"
                  checked={embeddingsEnabled}
                  onCheckedChange={(value) => updateAi({ embeddings_enabled: value === true })}
                />
                <FieldLabel htmlFor="enable-embeddings">Enable embeddings</FieldLabel>
              </Field>
              <FieldDescription>
                Embeddings always use <code>{EMBED_MODEL}</code> via Ollama.
              </FieldDescription>
            </FieldSet>
          </FieldSet>
          {/* UI */}
          <FieldSet>
            <FieldLegend>UI</FieldLegend>
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
            {/* Disable Keyboard Shortcut Hints */}
            <Field orientation="horizontal">
              <Checkbox
                id="disable-keyboard-shortcut-hints"
                name="disable-keyboard-shortcut-hints"
                className="h-4 w-4"
                checked={config?.settings?.disable_keyboard_shortcut_hints ?? false}
                onCheckedChange={(value) => {
                  if (!config) return

                  const disableKeyboardShortcutHints = value === true

                  setConfig({
                    ...config,
                    settings: {
                      ...(config.settings ?? {}),
                      disable_keyboard_shortcut_hints: disableKeyboardShortcutHints,
                    },
                  })

                  void saveConfig()
                }}
              />
              <FieldLabel>Disable Keyboard Shortcut Hints</FieldLabel>
            </Field>
            {/* Enable Debug Menu */}
            <Field orientation="horizontal">
              <Checkbox
                id="enable-debug-menu"
                name="enable-debug-menu"
                className="h-4 w-4"
                checked={config?.settings?.enable_debug_menu ?? false}
                onCheckedChange={(value) => {
                  if (!config) return

                  const enableDebugMenu = value === true

                  setConfig({
                    ...config,
                    settings: {
                      ...(config.settings ?? {}),
                      enable_debug_menu: enableDebugMenu,
                    },
                  })

                  void saveConfig()
                }}
              />
              <FieldLabel htmlFor="enable-debug-menu">Enable Debug Menu</FieldLabel>
            </Field>
          </FieldSet>
        </div>
        <DialogFooter className="sm:justify-start">
          <Button variant="outline">
            <a
              href="https://github.com/RaptorAssassin/smart-file-search"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2"
            >
              <SiGithub className="size-5" />
              View repo on GitHub
            </a>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
