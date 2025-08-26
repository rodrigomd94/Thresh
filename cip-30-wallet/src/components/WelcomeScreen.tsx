import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Loader2, Wallet, Plus } from 'lucide-react';
import { MnemonicGeneration } from './MnemonicGeneration';
import { WalletView } from './WalletView';

interface WalletMetadata {
  wallet_id: string;
  name: string;
  created_at: number;
}

type Screen = 'loading' | 'welcome' | 'create-wallet' | 'wallet-list' | 'wallet-view';

export function WelcomeScreen() {
  const [screen, setScreen] = useState<Screen>('loading');
  const [wallets, setWallets] = useState<WalletMetadata[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [selectedWallet, setSelectedWallet] = useState<WalletMetadata | null>(null);

  useEffect(() => {
    checkWalletStatus();
  }, []);

  const checkWalletStatus = async () => {
    try {
      setError(null);
      
      // Check if any wallets exist
      const hasWallet = await invoke<boolean>('check_wallet_exists');
      
      if (hasWallet) {
        // Load wallet list
        const walletList = await invoke<WalletMetadata[]>('list_wallets');
        setWallets(walletList);
        setScreen('wallet-list');
      } else {
        setScreen('welcome');
      }
    } catch (err) {
      console.error('Failed to check wallet status:', err);
      setError('Failed to check wallet status. Please try again.');
      setScreen('welcome');
    }
  };

  const handleCreateWallet = () => {
    setScreen('create-wallet');
  };

  const handleBackToWelcome = () => {
    setScreen('welcome');
  };

  const handleWalletCreated = () => {
    // Refresh the wallet list after creation
    checkWalletStatus();
  };

  const handleOpenWallet = (wallet: WalletMetadata) => {
    setSelectedWallet(wallet);
    setScreen('wallet-view');
  };

  const handleBackToList = () => {
    setSelectedWallet(null);
    checkWalletStatus();
  };

  if (screen === 'loading') {
    return (
      <div className="flex items-center justify-center min-h-screen ">
        <Card className="w-full max-w-md">
          <CardContent className="flex items-center justify-center p-8">
            <Loader2 className="h-8 w-8 animate-spin text-blue-600" />
            <span className="ml-2 text-sm text-gray-600">Loading wallet...</span>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (screen === 'create-wallet') {
    return (
      <MnemonicGeneration 
        onBack={handleBackToWelcome}
        onWalletCreated={handleWalletCreated}
      />
    );
  }

  if (screen === 'wallet-view' && selectedWallet) {
    return (
      <WalletView
        walletId={selectedWallet.wallet_id}
        walletName={selectedWallet.name}
        onBack={handleBackToList}
      />
    );
  }

  if (screen === 'wallet-list') {
    return (
      <div className="flex items-center justify-center min-h-screen ">
        <Card className="w-full max-w-2xl">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Wallet className="h-5 w-5" />
              Your Wallets
            </CardTitle>
            <CardDescription>
              Select a wallet to access or create a new one
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {wallets.map((wallet) => (
              <Card key={wallet.wallet_id} className="cursor-pointer hover:shadow-md transition-shadow">
                <CardContent className="flex items-center justify-between p-4">
                  <div>
                    <h3 className="font-medium">{wallet.name}</h3>
                    <p className="text-sm text-gray-500">
                      Created {new Date(wallet.created_at * 1000).toLocaleDateString()}
                    </p>
                  </div>
                  <Button 
                    variant="outline" 
                    size="sm"
                    onClick={() => handleOpenWallet(wallet)}
                  >
                    Open Wallet
                  </Button>
                </CardContent>
              </Card>
            ))}
            
            <Card className="border-2 border-dashed border-gray-300 cursor-pointer hover:border-blue-400 transition-colors">
              <CardContent 
                className="flex items-center justify-center p-8"
                onClick={handleCreateWallet}
              >
                <div className="text-center">
                  <Plus className="h-8 w-8 text-gray-400 mx-auto mb-2" />
                  <p className="text-sm text-gray-600">Create New Wallet</p>
                </div>
              </CardContent>
            </Card>
          </CardContent>
        </Card>
      </div>
    );
  }

  // Welcome screen for first-time users
  return (
    <div className="flex items-center justify-center min-h-screen ">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="mx-auto mb-4 h-12 w-12 rounded-full flex items-center justify-center">
            <Wallet className="h-6 w-6 text-blue-600" />
          </div>
          <CardTitle>Welcome to CIP-30 Wallet</CardTitle>
          <CardDescription>
            Your secure Cardano wallet powered by Tauri
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {error && (
            <div className="p-3 rounded-md bg-red-50 border border-red-200">
              <p className="text-sm text-red-600">{error}</p>
            </div>
          )}
          
          <Button 
            onClick={handleCreateWallet}
            className="w-full"
            size="lg"
          >
            <Plus className="h-4 w-4 mr-2" />
            Create New Wallet
          </Button>
          
          <p className="text-xs text-center text-gray-500">
            Your wallet data is encrypted and stored locally on your device
          </p>
        </CardContent>
      </Card>
    </div>
  );
}