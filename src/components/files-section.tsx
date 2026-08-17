import { useCallback, useEffect, useRef, type ReactNode } from 'react'
import { motion, type Variants } from 'framer-motion'
import { commands, type SearchResult } from '@/bindings/bindings'
import { useSearchStore } from '@/stores/search-store'
import { formatBytes } from '@/lib/utils'
import { File, Loader2Icon } from 'lucide-react'
import {
  SiAndroid,
  SiApacheparquet,
  SiAssemblyscript,
  SiBlender,
  SiC,
  SiClojure,
  SiCmake,
  SiCplusplus,
  SiCrystal,
  SiCss,
  SiDart,
  SiDocker,
  SiDotenv,
  SiDotnet,
  SiDuckdb,
  SiElixir,
  SiElm,
  SiErlang,
  SiFigma,
  SiFortran,
  SiFsharp,
  SiGimp,
  SiGnubash,
  SiGo,
  SiGodotengine,
  SiGradle,
  SiGraphql,
  SiHaskell,
  SiHaxe,
  SiHtml5,
  SiJavascript,
  SiJpeg,
  SiJulia,
  SiJupyter,
  SiKotlin,
  SiLatex,
  SiLess,
  SiLua,
  SiLuau,
  SiMarkdown,
  SiMdx,
  SiNim,
  SiNixos,
  SiOpenjdk,
  SiPerl,
  SiPhp,
  SiPython,
  SiR,
  SiReact,
  SiRenpy,
  SiRuby,
  SiRust,
  SiSass,
  SiScala,
  SiSolidity,
  SiSqlite,
  SiSvelte,
  SiSvg,
  SiSwift,
  SiTerraform,
  SiToml,
  SiTypescript,
  SiUnrealengine,
  SiVim,
  SiVuedotjs,
  SiYaml,
  SiZig,
  type IconType,
} from '@icons-pack/react-simple-icons'

const FILE_LIMIT = 100

const listVariants: Variants = {
  hidden: {},
  show: {
    transition: { staggerChildren: 0.06 },
  },
}

const itemVariants: Variants = {
  hidden: { opacity: 0, y: 24 },
  show: { opacity: 1, y: 0, transition: { duration: 0.35, ease: 'easeOut' } },
}

export default function FilesSection() {
  const searchQuery = useSearchStore((state) => state.searchQuery)
  const files = useSearchStore((state) => state.files)
  const setFiles = useSearchStore((state) => state.setFiles)
  const isSearching = useSearchStore((state) => state.isSearching)
  const setIsSearching = useSearchStore((state) => state.setIsSearching)
  const scrollRef = useRef<HTMLDivElement>(null)

  const searchFiles = useCallback(async (query: string) => {
    const result = await commands.searchFiles(query, null, FILE_LIMIT)
    return result.status === 'ok' ? result.data.results : []
  }, [])

  useEffect(() => {
    let cancelled = false
    setIsSearching(true)
    void searchFiles(searchQuery ?? '').then((files) => {
      if (!cancelled) {
        setFiles(files)
        setIsSearching(false)
      }
    })
    return () => {
      cancelled = true
    }
  }, [searchQuery, searchFiles, setFiles, setIsSearching])

  const isEmpty = !searchQuery || searchQuery.trim() === ''

  return (
    <div ref={scrollRef} className="scrollbar-hidden h-full overflow-y-auto">
      {isSearching ? (
        <EmptyState>
          <Loader2Icon className="size-5 animate-spin" />
          <span>Searching...</span>
        </EmptyState>
      ) : isEmpty ? (
        <EmptyState>Type a search to find files</EmptyState>
      ) : files.length === 0 ? (
        <EmptyState>No results</EmptyState>
      ) : (
        <motion.ul
          key={searchQuery}
          variants={listVariants}
          initial="hidden"
          whileInView="show"
          viewport={{ root: scrollRef, once: true }}
          className="flex flex-col gap-2 p-4"
        >
          {files.map((file) => (
            <motion.li key={file.file_id} variants={itemVariants}>
              <FileCard file={file} />
            </motion.li>
          ))}
        </motion.ul>
      )}
    </div>
  )
}

function EmptyState({ children }: { children: ReactNode }) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 p-4 text-sm text-muted-foreground">
      {children}
    </div>
  )
}

function FileCard({ file }: { file: SearchResult }) {
  return (
    <button
      type="button"
      onClick={() => void commands.revealInFolder(file.file_path)}
      className="flex w-full items-center gap-3 rounded-lg border border-border bg-background p-3 text-left transition-colors hover:bg-muted"
    >
      <FileIcon file={file} />
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="truncate text-sm font-medium">{file.file_name}</span>
        <span className="truncate text-xs text-muted-foreground">{file.file_path}</span>
      </span>
      <span className="flex shrink-0 flex-col items-end gap-1">
        <span className="text-xs text-muted-foreground">{formatBytes(file.file_size)}</span>
        <span className="text-xs text-muted-foreground">
          {file.modified_at?.slice(0, 10) ?? ''}
        </span>
      </span>
    </button>
  )
}

const FILE_ICON_MAP: Record<string, IconType> = {
  js: SiJavascript,
  mjs: SiJavascript,
  cjs: SiJavascript,
  ts: SiTypescript,
  jsx: SiReact,
  tsx: SiReact,
  html: SiHtml5,
  htm: SiHtml5,
  css: SiCss,
  scss: SiSass,
  sass: SiSass,
  less: SiLess,
  svg: SiSvg,
  yaml: SiYaml,
  yml: SiYaml,
  toml: SiToml,
  graphql: SiGraphql,
  gql: SiGraphql,
  mdx: SiMdx,
  svelte: SiSvelte,
  vue: SiVuedotjs,
  jpg: SiJpeg,
  jpeg: SiJpeg,

  py: SiPython,
  pyw: SiPython,
  rb: SiRuby,
  php: SiPhp,
  go: SiGo,
  rs: SiRust,
  c: SiC,
  h: SiC,
  cpp: SiCplusplus,
  cc: SiCplusplus,
  cxx: SiCplusplus,
  hpp: SiCplusplus,
  cs: SiDotnet,
  java: SiOpenjdk,
  kt: SiKotlin,
  kts: SiKotlin,
  swift: SiSwift,
  dart: SiDart,
  lua: SiLua,
  luau: SiLuau,
  r: SiR,
  scala: SiScala,
  clj: SiClojure,
  cljs: SiClojure,
  cljc: SiClojure,
  hs: SiHaskell,
  elm: SiElm,
  nim: SiNim,
  cr: SiCrystal,
  zig: SiZig,
  ex: SiElixir,
  exs: SiElixir,
  erl: SiErlang,
  fs: SiFsharp,
  fsl: SiFsharp,
  fsx: SiFsharp,
  pl: SiPerl,
  pm: SiPerl,
  sol: SiSolidity,
  as: SiAssemblyscript,
  f90: SiFortran,
  f95: SiFortran,
  rpy: SiRenpy,
  jl: SiJulia,
  hx: SiHaxe,

  md: SiMarkdown,
  markdown: SiMarkdown,
  tex: SiLatex,
  bib: SiLatex,
  ipynb: SiJupyter,

  sh: SiGnubash,
  bash: SiGnubash,
  dockerfile: SiDocker,
  nix: SiNixos,
  tf: SiTerraform,
  tfvars: SiTerraform,
  gradle: SiGradle,
  cmake: SiCmake,
  env: SiDotenv,

  vim: SiVim,

  sql: SiSqlite,
  sqlite: SiSqlite,
  sqlite3: SiSqlite,
  db: SiSqlite,
  duckdb: SiDuckdb,
  parquet: SiApacheparquet,

  blend: SiBlender,
  blend1: SiBlender,
  fig: SiFigma,
  xcf: SiGimp,
  gd: SiGodotengine,
  tscn: SiGodotengine,
  uasset: SiUnrealengine,
  apk: SiAndroid,
}

const FILE_COLOR_MAP: Record<string, string> = {
  js: '#F7DF1E',
  mjs: '#F7DF1E',
  cjs: '#F7DF1E',
  ts: '#3178C6',
  jsx: '#61DAFB',
  tsx: '#61DAFB',
  html: '#E34F26',
  htm: '#E34F26',
  css: '#663399',
  scss: '#CC6699',
  sass: '#CC6699',
  less: '#1D365D',
  svg: '#FFB13B',
  yaml: '#CB171E',
  yml: '#CB171E',
  toml: '#9C4121',
  graphql: '#E10098',
  gql: '#E10098',
  mdx: '#1B1F24',
  svelte: '#FF3E00',
  vue: '#4FC08D',
  jpg: '#8A8A8A',
  jpeg: '#8A8A8A',

  py: '#3776AB',
  pyw: '#3776AB',
  rb: '#CC342D',
  php: '#777BB4',
  go: '#00ADD8',
  rs: '#000000',
  c: '#A8B9CC',
  h: '#A8B9CC',
  cpp: '#00599C',
  cc: '#00599C',
  cxx: '#00599C',
  hpp: '#00599C',
  cs: '#512BD4',
  java: '#000000',
  kt: '#7F52FF',
  kts: '#7F52FF',
  swift: '#F05138',
  dart: '#0175C2',
  lua: '#000080',
  luau: '#00A2FF',
  r: '#276DC3',
  scala: '#DC322F',
  clj: '#5881D8',
  cljs: '#5881D8',
  cljc: '#5881D8',
  hs: '#5D4F85',
  elm: '#1293D8',
  nim: '#FFE953',
  cr: '#000000',
  zig: '#F7A41D',
  ex: '#4B275F',
  exs: '#4B275F',
  erl: '#A90533',
  fs: '#378BBA',
  fsl: '#378BBA',
  fsx: '#378BBA',
  pl: '#0073A1',
  pm: '#0073A1',
  sol: '#363636',
  as: '#007ACC',
  f90: '#734F96',
  f95: '#734F96',
  rpy: '#FF7F7F',
  jl: '#9558B2',
  hx: '#EA8220',

  md: '#000000',
  markdown: '#000000',
  tex: '#008080',
  bib: '#008080',
  ipynb: '#F37626',

  sh: '#4EAA25',
  bash: '#4EAA25',
  dockerfile: '#2496ED',
  nix: '#5277C3',
  tf: '#844FBA',
  tfvars: '#844FBA',
  gradle: '#02303A',
  cmake: '#064F8C',
  env: '#ECD53F',

  vim: '#019733',

  sql: '#003B57',
  sqlite: '#003B57',
  sqlite3: '#003B57',
  db: '#003B57',
  duckdb: '#FFF000',
  parquet: '#50ABF1',

  blend: '#E87D0D',
  blend1: '#E87D0D',
  fig: '#F24E1E',
  xcf: '#8C8073',
  gd: '#478CBF',
  tscn: '#478CBF',
  uasset: '#0E1128',
  apk: '#3DDC84',
}

function FileIcon({ file }: { file: SearchResult }) {
  const extension = file.extension.toLowerCase()
  const Icon = FILE_ICON_MAP[extension]
  const color = FILE_COLOR_MAP[extension]

  if (Icon && color) {
    return (
      <span
        className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md"
        style={{ backgroundColor: `${color}1A`, color }}
      >
        <Icon size={20} />
      </span>
    )
  }

  if (!extension) {
    return (
      <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-accent/10 text-accent">
        <File className="size-5" />
      </span>
    )
  }

  return (
    <span className="flex h-10 w-10 shrink-0 select-none items-center justify-center overflow-visible rounded-md bg-accent/10 text-[13px] font-bold leading-none text-accent">
      {extension.toUpperCase().slice(0, 4)}
    </span>
  )
}
