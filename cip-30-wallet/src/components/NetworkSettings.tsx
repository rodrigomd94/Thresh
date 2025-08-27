import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { AlertTriangle, Settings, Wifi, Save, RotateCcw, Check } from 'lucide-react';

interface NetworkInfo {
  active: string;
  config_default: string;
  has_runtime_override: boolean;
}

interface NetworkSettingsProps {
  onClose: () => void;
  onNetworkChanged?: () => void;
}

export function NetworkSettings({ onClose, onNetworkChanged }: NetworkSettingsProps) {
  const [networkInfo, setNetworkInfo] = useState<NetworkInfo | null>(null);
  const [selectedNetwork, setSelectedNetwork] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [applying, setApplying] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    loadNetworkInfo();
  }, []);

  const loadNetworkInfo = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const info = await invoke<NetworkInfo>('get_network_info');
      setNetworkInfo(info);
      setSelectedNetwork(info.active);
    } catch (err) {
      console.error('Failed to load network info:', err);
      setError('Failed to load network information');
    } finally {
      setLoading(false);
    }
  };

  const applyNetwork = async () => {
    if (!selectedNetwork || selectedNetwork === networkInfo?.active) return;

    try {
      setApplying(true);
      setError(null);
      setSuccess(null);

      await invoke('set_runtime_network', { network: selectedNetwork });
      await loadNetworkInfo(); // Refresh info
      setSuccess(`Network switched to ${selectedNetwork.toUpperCase()}`);
      onNetworkChanged?.();
    } catch (err) {
      console.error('Failed to set network:', err);
      setError('Failed to switch network');
    } finally {
      setApplying(false);
    }
  };

  const saveAsDefault = async () => {
    if (!selectedNetwork) return;

    try {
      setSaving(true);
      setError(null);
      setSuccess(null);

      await invoke('save_network_to_config', { network: selectedNetwork });
      await loadNetworkInfo(); // Refresh info
      setSuccess(`${selectedNetwork.toUpperCase()} saved as default network`);
    } catch (err) {
      console.error('Failed to save network config:', err);
      setError('Failed to save network configuration');
    } finally {
      setSaving(false);
    }
  };

  const resetToDefault = async () => {
    try {
      setError(null);
      setSuccess(null);

      await invoke('reset_runtime_network');
      await loadNetworkInfo(); // Refresh info
      setSelectedNetwork(networkInfo?.config_default || 'mainnet');
      setSuccess('Reset to configuration default');
      onNetworkChanged?.();
    } catch (err) {
      console.error('Failed to reset network:', err);
      setError('Failed to reset to default');
    }
  };

  if (loading) {
    return (
      <Card className="w-full max-w-md">
        <CardContent className="flex items-center justify-center p-8">
          <div className="text-center">
            <Settings className="h-8 w-8 animate-spin mx-auto mb-2" />
            <p>Loading network settings...</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  const hasChanges = selectedNetwork !== networkInfo?.active;
  const isDefaultNetwork = selectedNetwork === networkInfo?.config_default;

  return (
    <Card className="w-full max-w-lg">
      <CardHeader>
        <div className="flex items-center gap-2">
          <Wifi className="h-5 w-5" />
          <div>
            <CardTitle>Network Settings</CardTitle>
            <CardDescription>
              Choose which Cardano network to use for wallet operations
            </CardDescription>
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-6">
        {error && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {success && (
          <Alert>
            <Check className="h-4 w-4" />
            <AlertDescription className="text-green-600">{success}</AlertDescription>
          </Alert>
        )}

        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium">Current Network</h3>
            <div className="flex items-center gap-2">
              <Badge variant={networkInfo?.active === 'mainnet' ? 'default' : 'secondary'}>
                {networkInfo?.active?.toUpperCase()}
              </Badge>
              {networkInfo?.has_runtime_override && (
                <Badge variant="outline" className="text-xs">
                  Override
                </Badge>
              )}
            </div>
          </div>

          <div className="space-y-3">
            <Label htmlFor="network-selection" className="text-sm font-medium">
              Select Network
            </Label>
            <RadioGroup
              value={selectedNetwork}
              onValueChange={setSelectedNetwork}
              className="space-y-3"
            >
              <div className="flex items-center space-x-2">
                <RadioGroupItem value="mainnet" id="mainnet" />
                <Label htmlFor="mainnet" className="flex-1">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium">Mainnet</div>
                      <div className="text-sm text-muted-foreground">
                        Production Cardano network with real ADA
                      </div>
                    </div>
                    {networkInfo?.config_default === 'mainnet' && (
                      <Badge variant="outline" className="text-xs">Default</Badge>
                    )}
                  </div>
                </Label>
              </div>

              <div className="flex items-center space-x-2">
                <RadioGroupItem value="testnet" id="testnet" />
                <Label htmlFor="testnet" className="flex-1">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium">Testnet</div>
                      <div className="text-sm text-muted-foreground">
                        Test network for development and testing
                      </div>
                    </div>
                    {networkInfo?.config_default === 'testnet' && (
                      <Badge variant="outline" className="text-xs">Default</Badge>
                    )}
                  </div>
                </Label>
              </div>
            </RadioGroup>
          </div>
        </div>

        {hasChanges && (
          <Alert>
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              <strong>Warning:</strong> Changing networks will affect address generation and 
              wallet operations. Make sure you're using the correct network for your needs.
            </AlertDescription>
          </Alert>
        )}

        <div className="flex flex-col gap-2">
          <div className="flex gap-2">
            <Button 
              onClick={applyNetwork}
              disabled={!hasChanges || applying}
              className="flex-1"
            >
              {applying ? 'Applying...' : 'Apply Network'}
            </Button>
            
            {networkInfo?.has_runtime_override && (
              <Button
                variant="outline"
                onClick={resetToDefault}
                className="flex items-center gap-1"
              >
                <RotateCcw className="h-4 w-4" />
                Reset
              </Button>
            )}
          </div>

          <div className="flex gap-2">
            <Button
              variant="outline"
              onClick={saveAsDefault}
              disabled={isDefaultNetwork || saving}
              className="flex-1 flex items-center gap-1"
            >
              <Save className="h-4 w-4" />
              {saving ? 'Saving...' : 'Save as Default'}
            </Button>

            <Button variant="outline" onClick={onClose}>
              Close
            </Button>
          </div>
        </div>

        <div className="text-xs text-muted-foreground space-y-1">
          <p>• <strong>Apply:</strong> Switch network for this session</p>
          <p>• <strong>Save as Default:</strong> Set as startup network</p>
          <p>• <strong>Reset:</strong> Return to configuration default</p>
        </div>
      </CardContent>
    </Card>
  );
}