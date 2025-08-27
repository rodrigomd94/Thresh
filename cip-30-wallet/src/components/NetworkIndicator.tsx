import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Wifi, Settings } from 'lucide-react';

interface NetworkIndicatorProps {
  onOpenSettings?: () => void;
  refreshTrigger?: number; // Used to trigger refresh when network changes
}

export function NetworkIndicator({ onOpenSettings, refreshTrigger }: NetworkIndicatorProps) {
  const [network, setNetwork] = useState<string>('mainnet');
  const [hasOverride, setHasOverride] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadNetworkInfo();
  }, [refreshTrigger]);

  const loadNetworkInfo = async () => {
    try {
      const info = await invoke<{
        active: string;
        config_default: string;
        has_runtime_override: boolean;
      }>('get_network_info');
      
      setNetwork(info.active);
      setHasOverride(info.has_runtime_override);
    } catch (err) {
      console.error('Failed to load network info:', err);
      // Fallback to current network command
      try {
        const currentNetwork = await invoke<string>('get_current_network');
        setNetwork(currentNetwork);
      } catch {
        setNetwork('mainnet'); // Ultimate fallback
      }
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <Badge variant="outline" className="text-xs">
        <Wifi className="h-3 w-3 mr-1 animate-pulse" />
        Loading...
      </Badge>
    );
  }

  return (
    <div className="flex items-center gap-2">
      <Badge 
        variant={network === 'mainnet' ? 'default' : 'secondary'}
        className="text-xs"
      >
        <Wifi className="h-3 w-3 mr-1" />
        {network.toUpperCase()}
        {hasOverride && ' *'}
      </Badge>
      
      {onOpenSettings && (
        <Button
          variant="ghost" 
          size="sm"
          onClick={onOpenSettings}
          className="h-6 w-6 p-0"
          title="Network Settings"
        >
          <Settings className="h-3 w-3" />
        </Button>
      )}
    </div>
  );
}