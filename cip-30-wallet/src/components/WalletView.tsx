import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Copy, Wallet, RefreshCw, ArrowLeft } from 'lucide-react';

interface AddressInfo {
  address: string;
  path: string;
  account_index: number;
  address_index: number;
}

interface WalletViewProps {
  walletId: string;
  walletName: string;
  onBack: () => void;
  isNewWallet?: boolean;
}

export function WalletView({ walletId, walletName, onBack, isNewWallet = false }: WalletViewProps) {
  const [addresses, setAddresses] = useState<AddressInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [copiedAddress, setCopiedAddress] = useState<string | null>(null);

  useEffect(() => {
    loadAddresses();
  }, [walletId]);

  const loadAddresses = async () => {
    try {
      setLoading(true);
      setError(null);

      // Load first 5 addresses from account 0 using public key derivation
      const addressList = await invoke<AddressInfo[]>('get_addresses_from_wallet', {
        walletId,
        accountIndex: 0,
        count: 5,
      });

      setAddresses(addressList);
    } catch (err) {
      console.error('Failed to load addresses:', err);
      setError('Failed to load wallet addresses. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedAddress(text);
      setTimeout(() => setCopiedAddress(null), 2000);
    } catch (err) {
      console.error('Failed to copy to clipboard:', err);
    }
  };

  const formatAddress = (address: string) => {
    if (address.length > 20) {
      return `${address.slice(0, 36)}...${address.slice(-16)}`;
    }
    return address;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <Card className="w-full max-w-md">
          <CardContent className="flex flex-col items-center justify-center p-8">
            <RefreshCw className="h-12 w-12 animate-spin text-primary mb-4" />
            <h3 className="text-lg font-medium mb-2">Loading Wallet...</h3>
            <p className="text-sm text-center text-muted-foreground">
              {isNewWallet ? 'Generating addresses from your recovery phrase' : 'Generating addresses from your wallet'}
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="flex items-center justify-center min-h-screen p-4">
      <Card className="w-full max-w-4xl">
        <CardHeader>
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="sm" onClick={onBack}>
              <ArrowLeft className="h-4 w-4" />
            </Button>
            <div className="flex-1">
              <CardTitle className="flex items-center gap-2">
                <Wallet className="h-5 w-5" />
                {walletName}
              </CardTitle>
              <CardDescription>
                {isNewWallet ? 'Your Cardano wallet is ready! Here are your first addresses.' : 'Your Cardano wallet addresses (public key derivation only)'}
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        
        <CardContent className="space-y-6">
          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          <Alert>
            <Wallet className="h-4 w-4" />
            <AlertDescription>
              {isNewWallet ? (
                <>
                  <strong>Wallet Created Successfully!</strong> Your addresses are derived from your recovery phrase. 
                  You can share these addresses to receive ADA.
                </>
              ) : (
                <>
                  <strong>Secure Address Display!</strong> These addresses are derived using only your public key. 
                  No private key is needed to view your receiving addresses.
                </>
              )}
            </AlertDescription>
          </Alert>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-lg font-medium">Receiving Addresses</h3>
              <Badge variant="secondary">Account 0</Badge>
            </div>

            <div className="grid gap-3">
              {addresses.map((addressInfo, index) => (
                <Card key={index} className="transition-all hover:shadow-md">
                  <CardContent className="p-4">
                    <div className="flex items-center justify-between">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          <Badge variant="outline" className="text-xs">
                            Address {index + 1}
                          </Badge>
                          <Badge variant="outline" className="text-xs font-mono">
                            {addressInfo.path}
                          </Badge>
                        </div>
                        
                        <div className="font-mono text-sm bg-muted p-2 rounded border">
                          <span className="">{formatAddress(addressInfo.address)}</span>
                        </div>
                      </div>
                      
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => copyToClipboard(addressInfo.address)}
                        className="ml-2"
                      >
                        <Copy className="h-4 w-4" />
                        {copiedAddress === addressInfo.address ? 'Copied!' : 'Copy'}
                      </Button>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>

            <Card className="border-dashed">
              <CardContent className="p-4 text-center">
                <p className="text-sm text-muted-foreground mb-2">
                  Need more addresses?
                </p>
                <Button variant="outline" size="sm" onClick={loadAddresses}>
                  <RefreshCw className="h-4 w-4 mr-2" />
                  Generate More
                </Button>
              </CardContent>
            </Card>
          </div>

          <div className="pt-4 border-t">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-center">
              <div>
                <h4 className="font-medium">Balance</h4>
                <p className="text-2xl font-bold text-primary">0 ADA</p>
                <p className="text-xs text-muted-foreground">No transactions yet</p>
              </div>
              <div>
                <h4 className="font-medium">Addresses</h4>
                <p className="text-2xl font-bold text-primary">{addresses.length}</p>
                <p className="text-xs text-muted-foreground">Generated</p>
              </div>
              <div>
                <h4 className="font-medium">Account</h4>
                <p className="text-2xl font-bold text-primary">0</p>
                <p className="text-xs text-muted-foreground">Default account</p>
              </div>
            </div>
          </div>

          <Alert>
            <AlertDescription className="text-xs">
              <strong>Security Note:</strong> {isNewWallet ? 'These addresses are generated from your recovery phrase. Keep your recovery phrase secure - anyone with access to it can control your wallet.' : 'This view uses only public key derivation. Your private keys remain encrypted and are only needed for transaction signing.'}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    </div>
  );
}