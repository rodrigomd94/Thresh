import { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ArrowLeft, CheckCircle, AlertTriangle, Shuffle } from 'lucide-react';

interface MnemonicConfirmationProps {
  originalMnemonic: string[];
  onBack: () => void;
  onConfirmed: () => void;
}

interface WordSlot {
  index: number;
  word: string;
  userInput: string;
  isCorrect: boolean;
}

export function MnemonicConfirmation({ originalMnemonic, onBack, onConfirmed }: MnemonicConfirmationProps) {
  const [confirmationWords, setConfirmationWords] = useState<WordSlot[]>([]);
  const [allCorrect, setAllCorrect] = useState(false);
  const [showResults, setShowResults] = useState(false);

  // Generate random word positions to verify (4-6 words)
  useEffect(() => {
    const numWordsToVerify = Math.min(6, Math.max(4, Math.floor(originalMnemonic.length / 4)));
    const randomIndices:any = [];
    
    while (randomIndices.length < numWordsToVerify) {
      const randomIndex = Math.floor(Math.random() * originalMnemonic.length);
      if (!randomIndices.includes(randomIndex)) {
        randomIndices.push(randomIndex);
      }
    }
    
    randomIndices.sort((a, b) => a - b);
    
    const words = randomIndices.map(index => ({
      index,
      word: originalMnemonic[index],
      userInput: '',
      isCorrect: false,
    }));
    
    setConfirmationWords(words);
  }, [originalMnemonic]);

  const handleInputChange = (slotIndex: number, value: string) => {
    const updatedWords = [...confirmationWords];
    updatedWords[slotIndex].userInput = value.trim().toLowerCase();
    updatedWords[slotIndex].isCorrect = updatedWords[slotIndex].userInput === updatedWords[slotIndex].word.toLowerCase();
    
    setConfirmationWords(updatedWords);
    
    // Check if all words are correct
    const allMatch = updatedWords.every(slot => slot.isCorrect);
    setAllCorrect(allMatch);
  };

  const handleVerify = () => {
    setShowResults(true);
    if (allCorrect) {
      setTimeout(() => onConfirmed(), 1500);
    }
  };

  const handleShuffle = () => {
    // Generate new random positions
    const numWordsToVerify = confirmationWords.length;
    const randomIndices:any = [];
    
    while (randomIndices.length < numWordsToVerify) {
      const randomIndex = Math.floor(Math.random() * originalMnemonic.length);
      if (!randomIndices.includes(randomIndex)) {
        randomIndices.push(randomIndex);
      }
    }
    
    randomIndices.sort((a, b) => a - b);
    
    const words = randomIndices.map(index => ({
      index,
      word: originalMnemonic[index],
      userInput: '',
      isCorrect: false,
    }));
    
    setConfirmationWords(words);
    setShowResults(false);
    setAllCorrect(false);
  };

  if (showResults && allCorrect) {
    return (
      <div className="flex items-center justify-center min-h-screen ">
        <Card className="w-full max-w-md">
          <CardContent className="flex flex-col items-center justify-center p-8">
            <CheckCircle className="h-16 w-16 text-primary mb-4" />
            <h3 className="text-xl font-medium mb-2">Perfect!</h3>
            <p className="text-sm text-muted-foreground text-center">
              Your recovery phrase has been confirmed successfully
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
              <CardTitle>Confirm Recovery Phrase</CardTitle>
              <CardDescription>
                Please enter the requested words from your recovery phrase to verify you've written them down
              </CardDescription>
            </div>
          </div>
        </CardHeader>
        
        <CardContent className="space-y-6">
          <Alert>
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              Enter the words exactly as they appeared in your recovery phrase (lowercase, no extra spaces)
            </AlertDescription>
          </Alert>

          <div className="grid grid-cols-1 gap-4">
            {confirmationWords.map((slot, slotIndex) => (
              <div key={slotIndex} className="space-y-2">
                <label className="text-sm font-medium">
                  Word #{slot.index + 1}
                </label>
                <div className="flex gap-2">
                  <Input
                    value={slot.userInput}
                    onChange={(e) => handleInputChange(slotIndex, e.target.value)}
                    placeholder={`Enter word #${slot.index + 1}`}
                    className={showResults ? (slot.isCorrect ? 'border-green-500' : 'border-red-500') : ''}
                  />
                  {showResults && (
                    <Badge variant={slot.isCorrect ? 'default' : 'destructive'}>
                      {slot.isCorrect ? '✓' : '✗'}
                    </Badge>
                  )}
                </div>
                {showResults && !slot.isCorrect && (
                  <p className="text-sm text-destructive">
                    Expected a different word.
                  </p>
                )}
              </div>
            ))}
          </div>

          {showResults && !allCorrect && (
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>
                Some words don't match. Please check your recovery phrase and try again.
              </AlertDescription>
            </Alert>
          )}

          <div className="flex gap-2">
            <Button onClick={handleShuffle} variant="outline">
              <Shuffle className="h-4 w-4 mr-2" />
              Different Words
            </Button>
            
            <Button 
              onClick={handleVerify}
              disabled={confirmationWords.some(slot => !slot.userInput)}
              className="flex-1"
            >
              {showResults ? 'Try Again' : 'Verify Words'}
            </Button>
          </div>

          <div className="text-center">
            <p className="text-xs text-muted-foreground">
              Verifying {confirmationWords.length} out of {originalMnemonic.length} words
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}