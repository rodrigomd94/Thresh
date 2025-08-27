import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ArrowLeft, RefreshCw, Eye, EyeOff, Check, AlertTriangle } from 'lucide-react';
import { MnemonicConfirmation } from './MnemonicConfirmation';
import { WalletView } from './WalletView';

interface MnemonicGenerationProps {
  onBack: () => void;
  onWalletCreated: () => void;
}

type Step = 'generate' | 'confirm' | 'password' | 'creating' | 'dashboard';

export function MnemonicGeneration({ onBack, onWalletCreated }: MnemonicGenerationProps) {
  const [step, setStep] = useState<Step>('generate');
  const [mnemonic, setMnemonic] = useState<string[]>([]);
  const [walletName, setWalletName] = useState('My Cardano Wallet');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showMnemonic, setShowMnemonic] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [walletId, setWalletId] = useState<string | null>(null);

  const generateMnemonic = async (wordCount: number = 24) => {
    try {
      setLoading(true);
      setError(null);
      
      const newMnemonic = await invoke<string[]>('generate_new_mnemonic', { wordCount });
      setMnemonic(newMnemonic);
    } catch (err) {
      console.error('Failed to generate mnemonic:', err);
      setError('Failed to generate mnemonic. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const handleConfirmMnemonic = () => {
    setStep('password');
  };

  const handleCreateWallet = async () => {
    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    if (password.length < 8) {
      setError('Password must be at least 8 characters long');
      return;
    }

    try {
      setStep('creating');
      setError(null);

      const response = await invoke<{wallet_id: string}>('create_wallet', {
        request: {
          name: walletName,
          password: password,
          mnemonic: mnemonic,
        }
      });

      // Store wallet ID for dashboard
      setWalletId(response.wallet_id);
      setStep('dashboard');
    } catch (err) {
      console.error('Failed to create wallet:', err);
      setError('Failed to create wallet. Please try again.');
      setStep('password');
    }
  };

  const getPasswordStrength = (pass: string) => {
    let score = 0;
    if (pass.length >= 8) score++;
    if (pass.length >= 12) score++;
    if (/[A-Z]/.test(pass)) score++;
    if (/[a-z]/.test(pass)) score++;
    if (/[0-9]/.test(pass)) score++;
    if (/[^A-Za-z0-9]/.test(pass)) score++;
    
    if (score <= 2) return { strength: 'Weak', color: 'bg-destructive' };
    if (score <= 4) return { strength: 'Medium', color: 'bg-primary/70' };
    return { strength: 'Strong', color: 'bg-primary' };
  };

  if (step === 'confirm') {
    return (
      <MnemonicConfirmation
        originalMnemonic={mnemonic}
        onBack={() => setStep('generate')}
        onConfirmed={handleConfirmMnemonic}
      />
    );
  }

  if (step === 'dashboard' && walletId) {
    return (
      <WalletView
        walletId={walletId}
        walletName={walletName}
        onBack={onWalletCreated}
        isNewWallet={true}
      />
    );
  }

  if (step === 'creating') {
    return (
      <div className="flex items-center justify-center min-h-screen ">
        <Card className="w-full max-w-md">
          <CardContent className="flex flex-col items-center justify-center p-8">
            <RefreshCw className="h-12 w-12 animate-spin text-blue-600 mb-4" />
            <h3 className="text-lg font-medium mb-2">Creating Wallet...</h3>
            <p className="text-sm text-center">
              Encrypting your wallet data with your password
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="flex items-center justify-center min-h-screen  p-4">
      <Card className="w-full max-w-2xl">
        <CardHeader>
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="sm" onClick={onBack}>
              <ArrowLeft className="h-4 w-4" />
            </Button>
            <div>
              <CardTitle>
                {step === 'generate' ? 'Generate Recovery Phrase' : 'Set Wallet Password'}
              </CardTitle>
              <CardDescription>
                {step === 'generate' 
                  ? 'Your recovery phrase is the key to your wallet. Keep it safe!'
                  : 'Create a strong password to encrypt your wallet'
                }
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

          {step === 'generate' && (
            <>
              <div className="space-y-4">
                <div>
                  <Label htmlFor="walletName">Wallet Name</Label>
                  <Input
                    id="walletName"
                    value={walletName}
                    onChange={(e) => setWalletName(e.target.value)}
                    placeholder="Enter wallet name"
                  />
                </div>

                <div className="flex gap-2">
                  <Button
                    onClick={() => generateMnemonic(12)}
                    disabled={loading}
                    variant="outline"
                  >
                    {loading ? <RefreshCw className="h-4 w-4 animate-spin" /> : '12 Words'}
                  </Button>
                  <Button
                    onClick={() => generateMnemonic(24)}
                    disabled={loading}
                  >
                    {loading ? <RefreshCw className="h-4 w-4 animate-spin" /> : '24 Words'}
                  </Button>
                </div>
              </div>

              {mnemonic.length > 0 && (
                <>
                  <div className="space-y-4">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-medium">Your Recovery Phrase</h3>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setShowMnemonic(!showMnemonic)}
                      >
                        {showMnemonic ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                        {showMnemonic ? 'Hide' : 'Show'}
                      </Button>
                    </div>

                    <div className="grid grid-cols-3 gap-2 p-4 rounded-lg">
                      {mnemonic.map((word, index) => (
                        <Badge
                          key={index}
                          variant="secondary"
                          className="justify-center bg-muted p-2 font-mono"
                        >
                          <span className="text-xs mr-1">{index + 1}.</span>
                          {showMnemonic ? word : '•••••'}
                        </Badge>
                      ))}
                    </div>

                    <Alert>
                      <AlertTriangle className="h-4 w-4" />
                      <AlertDescription>
                        <strong>Important:</strong> Write down these words in order and store them safely. 
                        Anyone with access to your recovery phrase can access your wallet.
                      </AlertDescription>
                    </Alert>

                    <div className="flex gap-2">
                      <Button onClick={() => generateMnemonic(mnemonic.length)} variant="outline">
                        <RefreshCw className="h-4 w-4 mr-2" />
                        Generate New
                      </Button>
                      <Button onClick={() => setStep('confirm')} className="flex-1">
                        I've Written It Down
                        <Check className="h-4 w-4 ml-2" />
                      </Button>
                    </div>
                  </div>
                </>
              )}
            </>
          )}

          {step === 'password' && (
            <div className="space-y-4">
              <div>
                <Label htmlFor="password">Password</Label>
                <Input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Enter a strong password"
                />
                {password && (
                  <div className="mt-2">
                    <div className="flex items-center gap-2 text-xs">
                      <div className="flex-1 bg-gray-200 rounded-full h-1">
                        <div 
                          className={`h-1 rounded-full transition-all ${getPasswordStrength(password).color}`}
                          style={{ width: `${(getPasswordStrength(password).strength === 'Weak' ? 33 : getPasswordStrength(password).strength === 'Medium' ? 66 : 100)}%` }}
                        />
                      </div>
                      <span className="">{getPasswordStrength(password).strength}</span>
                    </div>
                  </div>
                )}
              </div>

              <div>
                <Label htmlFor="confirmPassword">Confirm Password</Label>
                <Input
                  id="confirmPassword"
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  placeholder="Confirm your password"
                />
              </div>

              <Alert>
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  Your password encrypts your wallet on this device. If you forget it, 
                  you'll need your recovery phrase to restore access.
                </AlertDescription>
              </Alert>

              <div className="flex gap-2">
                <Button onClick={() => setStep('generate')} variant="outline">
                  Back
                </Button>
                <Button 
                  onClick={handleCreateWallet}
                  disabled={!password || !confirmPassword || password !== confirmPassword}
                  className="flex-1"
                >
                  Create Wallet
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}