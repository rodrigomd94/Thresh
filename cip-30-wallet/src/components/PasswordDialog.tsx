import React, { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Loader2 } from "lucide-react"

interface PasswordRequest {
  txId: string
  walletId: string
  txCbor: string
}

export function TransactionPasswordListener() {
  const [open, setOpen] = useState(false)
  const [password, setPassword] = useState("")
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [currentRequest, setCurrentRequest] = useState<PasswordRequest | null>(null)

  useEffect(() => {
    // Listen for password requests from the backend
    const unlisten = listen<PasswordRequest>("request-password", (event) => {
      console.log("Password request received:", event.payload)
      setCurrentRequest(event.payload)
      setOpen(true)
      setPassword("")
      setError(null)
    })

    return () => {
      unlisten.then(fn => fn())
    }
  }, [])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!currentRequest) return

    setError(null)
    setIsLoading(true)

    try {
      // First, validate the password by trying to load the wallet
      const isValid = await invoke<boolean>("validate_wallet_password", {
        walletId: currentRequest.walletId,
        password,
      })

      if (!isValid) {
        setError("Invalid password")
        return
      }

      // Submit the password to the backend
      await invoke("submit_transaction_password", {
        txId: currentRequest.txId,
        password: password,
      })

      // Clear sensitive data
      setPassword("")
      setCurrentRequest(null)
      setOpen(false)
    } catch (err) {
      console.error("Failed to submit password:", err)
      setError(err instanceof Error ? err.message : "Failed to validate password")
    } finally {
      setIsLoading(false)
    }
  }

  const handleCancel = async () => {
    if (currentRequest) {
      try {
        // Send cancellation to backend
        await invoke("submit_transaction_password", {
          txId: currentRequest.txId,
          password: null, // null indicates cancellation
        })
      } catch (err) {
        console.error("Failed to cancel transaction:", err)
      }
    }
    
    setPassword("")
    setError(null)
    setCurrentRequest(null)
    setOpen(false)
  }

  return (
    <Dialog open={open} onOpenChange={(newOpen) => {
      if (!newOpen) {
        handleCancel()
      }
    }}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Sign Transaction</DialogTitle>
          <DialogDescription>
            Enter your wallet password to sign this transaction.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="password">Password</Label>
            <Input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter wallet password"
              autoFocus
              disabled={isLoading}
              required
            />
          </div>
          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
          <DialogFooter className="gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={handleCancel}
              disabled={isLoading}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isLoading || !password}>
              {isLoading ? (
                <>
                  <Loader2 className="animate-spin" />
                  Validating...
                </>
              ) : (
                "Sign"
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}