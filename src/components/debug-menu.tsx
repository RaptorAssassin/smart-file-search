import { invoke } from '@tauri-apps/api/core';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { useEffect, useState } from 'react';
import { formatBytes } from '@/lib/utils';

type DebugInfo = {
  databasePath: string | null;
  databaseSize: number | null;
};

export default function DebugMenu({ openButton }: { openButton: React.ReactNode }) {
  const [debugInfo, setDebugInfo] = useState<DebugInfo>({
    databasePath: null,
    databaseSize: null,
  });
  useEffect(() => {
    const fetchDatabaseInfo = async () => {
      try {
        const path: string = await invoke('get_database_path');
        setDebugInfo((prev) => ({ ...prev, databasePath: path }));
        const size: number = await invoke('get_database_size');
        setDebugInfo((prev) => ({ ...prev, databaseSize: size }));
      } catch (error) {
        console.error('Error fetching database info', error);
      }
    };
    fetchDatabaseInfo();
  }, []);

  return (
    <Dialog>
      <DialogTrigger>{openButton}</DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle className="text-xl">Debug</DialogTitle>
        </DialogHeader>
        <h1 className="text-lg">Database</h1>
        <p>Database Path: {debugInfo.databasePath}</p>
        <p>Database Size: {formatBytes(debugInfo.databaseSize ?? 0)}</p>
      </DialogContent>
    </Dialog>
  );
}
