import { mnemonicToEntropy } from "./bip39";
import { getCardanoAddressInfo, getCardanoAddressUtxos } from "./cardano";
//import * as C from '../cml/libs/cardano_multiplatform_lib/cardano_multiplatform_lib.generated.js'
import { fromHex, toHex } from "./utils.js";
import { Network, Payload, SignedMessage } from "../types/cardano";
import { C, M } from "../cml/init.js";

import { Withdrawals, PrivateKey, Transaction, TransactionUnspentOutput, TransactionBody, NativeScripts } from "@libs/libs/cardano_multiplatform_lib/cardano_multiplatform_lib.generated.js";

/* import loader from "../cml/loader.js";
const C = loader.Cardano;
const M = loader.Message; */

export const testMenmonic = "evil nut entry pumpkin hour road own busy spirit initial code hair alert public liar"

export const createWalletFromMnemonic = async (mnemonic: string, network: Network) => {
    function harden(num: number): number {
        if (typeof num !== "number") throw new Error("Type number required here!");
        return 0x80000000 + num;
    }
    const entropy = await mnemonicToEntropy(mnemonic);
    const rootKey = C.Bip32PrivateKey.from_bip39_entropy(
        fromHex(entropy),
        new Uint8Array(),
    );

    const accountKey = rootKey.derive(harden(1852))
        .derive(harden(1815))
        .derive(harden(0)); //account index

    const pKey = accountKey.derive(0).derive(0).to_raw_key();
    const sKey = accountKey.derive(2).derive(0).to_raw_key();

    //const paymentKeyHash = pKey.to_public().hash();
    //const stakeKeyHash = sKey.to_public().hash();

    const { address, stakeAddress } = getAddressesFromKeys(pKey, sKey, network);

    return { address, stakeAddr:stakeAddress, paymentKey: pKey, stakeKey: sKey }
}

export const getKeyHashes = (paymentKey: PrivateKey, stakeKey: PrivateKey, tx: Transaction, utxos: TransactionUnspentOutput[]) => {
    const requiredKeyHashes = [];

    const paymentKeyHash = Buffer.from(
        paymentKey!.to_public().hash().to_bytes()
    ).toString('hex');
    /* const stakeKeyHash = Buffer.from(
        stakeKey!.to_public().hash().to_bytes()
    ).toString('hex'); */

    //get key hashes from inputs
    const inputs = tx.body().inputs();
    for (let i = 0; i < inputs.len(); i++) {
        const input = inputs.get(i);
        const txHash = Buffer.from(input.transaction_id().to_bytes()).toString(
            'hex'
        );
        const index = parseInt(input.index().to_str());
        if (
            utxos.some(
                (utxo) =>
                    Buffer.from(utxo.input().transaction_id().to_bytes()).toString(
                        'hex'
                    ) === txHash && parseInt(utxo.input().index().to_str()) === index
            )
        ) {
            requiredKeyHashes.push(paymentKeyHash);
        }
    }

    //get key hashes from certificates
    const txBody = tx.body();
    const certs = txBody.certs()
    const keyHashFromCert = (txBody: TransactionBody) => {
        for (let i = 0; i < certs!.len(); i++) {
            const cert = txBody.certs()!.get(i);
            if (cert) {
                if (cert.kind() === 0) {
                    const credential = cert.as_stake_registration()!.stake_credential();
                    if (credential.kind() === 0) {
                        // stake registration doesn't required key hash
                    }
                } else if (cert.kind() === 1) {
                    const credential = cert.as_stake_deregistration()!.stake_credential();
                    if (credential.kind() === 0) {
                        const keyHash = Buffer.from(
                            credential.to_keyhash()!.to_bytes()
                        ).toString('hex');
                        requiredKeyHashes.push(keyHash);
                    }
                } else if (cert.kind() === 2) {
                    const credential = cert.as_stake_delegation()!.stake_credential();
                    if (credential.kind() === 0) {
                        const keyHash = Buffer.from(
                            credential.to_keyhash()!.to_bytes()
                        ).toString('hex');
                        requiredKeyHashes.push(keyHash);
                    }
                } else if (cert.kind() === 3) {
                    const owners = cert
                        .as_pool_registration()!
                        .pool_params()
                        .pool_owners();
                    for (let i = 0; i < owners.len(); i++) {
                        const keyHash = Buffer.from(owners.get(i).to_bytes()).toString(
                            'hex'
                        );
                        requiredKeyHashes.push(keyHash);
                    }
                } else if (cert.kind() === 4) {
                    const operator = cert.as_pool_retirement()!.pool_keyhash().to_hex();
                    requiredKeyHashes.push(operator);
                } else if (cert.kind() === 6) {
                    const instant_reward = cert
                        .as_move_instantaneous_rewards_cert()!
                        .move_instantaneous_reward()!
                        .as_to_stake_creds()!
                        .keys();
                    for (let i = 0; i < instant_reward.len(); i++) {
                        const credential = instant_reward.get(i);

                        if (credential.kind() === 0) {
                            const keyHash = Buffer.from(
                                credential.to_keyhash()!.to_bytes()
                            ).toString('hex');
                            requiredKeyHashes.push(keyHash);
                        }
                    }
                }
            }

        }
    };
    if (certs) {
        if (certs) keyHashFromCert(txBody);
    }


    // key hashes from withdrawals
    const withdrawals = txBody.withdrawals();
    const keyHashFromWithdrawal = (withdrawals: Withdrawals) => {
        const rewardAddresses = withdrawals.keys();
        for (let i = 0; i < rewardAddresses.len(); i++) {
            const credential = rewardAddresses.get(i).payment_cred();
            if (credential.kind() === 0) {
                requiredKeyHashes.push(credential.to_keyhash()!.to_hex());
            }
        }
    };
    if (withdrawals) keyHashFromWithdrawal(withdrawals);

    //get key hashes from scripts
    const scripts = tx.witness_set().native_scripts();
    const keyHashFromScript: any = (scripts: NativeScripts) => {
        for (let i = 0; i < scripts.len(); i++) {
            const script = scripts.get(i);
            if (script.kind() === 0) {
                const keyHash = Buffer.from(
                    script.as_script_pubkey()!.addr_keyhash().to_bytes()
                ).toString('hex');
                requiredKeyHashes.push(keyHash);
            }
            if (script.kind() === 1) {
                return keyHashFromScript(script.as_script_all()!.native_scripts());
            }
            if (script.kind() === 2) {
                return keyHashFromScript(script.as_script_any()!.native_scripts());
            }
            if (script.kind() === 3) {
                return keyHashFromScript(script.as_script_n_of_k()!.native_scripts());
            }
        }
    };

    if (scripts) keyHashFromScript(scripts);

    //get keyHashes from required signers
    const requiredSigners = tx.body().required_signers();
    if (requiredSigners) {
        for (let i = 0; i < requiredSigners.len(); i++) {
            requiredKeyHashes.push(
                Buffer.from(requiredSigners.get(i).to_bytes()).toString('hex')
            );
        }
    }
    return requiredKeyHashes
}

export const getAddressesFromKeys = (paymentKey: PrivateKey, stakeKey: PrivateKey, network: Network) => {
    const paymentKeyHash = paymentKey.to_public().hash();
    const stakeKeyHash = stakeKey.to_public().hash();

    console.log({paymentKeyHash, stakeKeyHash})

    const networkId = network === "Mainnet" ? 1 : 0;
    console.log({networkId})
    const address = C.BaseAddress.new(
        networkId,
        C.StakeCredential.from_keyhash(paymentKeyHash),
        C.StakeCredential.from_keyhash(stakeKeyHash),
    ).to_address().to_bech32(undefined)

    console.log({address})
    const stakeAddr = C.RewardAddress.new(
        networkId,
        C.StakeCredential.from_keyhash(stakeKeyHash),
    ).to_address().to_bech32(undefined)
    return { address, stakeAddress: stakeAddr }
}

export function signDataUtil(
    addressHex: string,
    payload: Payload,
    privateKey: string,
): SignedMessage {
    const protectedHeaders = M.HeaderMap.new();
    protectedHeaders.set_algorithm_id(
        M.Label.from_algorithm_id(
            M.AlgorithmId.EdDSA,
        ),
    );
    protectedHeaders.set_header(
        M.Label.new_text("address"),
        M.CBORValue.new_bytes(fromHex(addressHex)),
    );
    const protectedSerialized = M.ProtectedHeaderMap.new(
        protectedHeaders,
    );
    const unprotectedHeaders = M.HeaderMap.new();
    const headers = M.Headers.new(
        protectedSerialized,
        unprotectedHeaders,
    );
    const builder = M.COSESign1Builder.new(
        headers,
        fromHex(payload),
        false,
    );
    const toSign = builder.make_data_to_sign().to_bytes();

    const priv = C.PrivateKey.from_bech32(privateKey);

    const signedSigStruc = priv.sign(toSign).to_bytes();
    const coseSign1 = builder.build(signedSigStruc);

    const key = M.COSEKey.new(
        M.Label.from_key_type(M.KeyType.OKP), //OKP
    );
    key.set_algorithm_id(
        M.Label.from_algorithm_id(
            M.AlgorithmId.EdDSA,
        ),
    );
    key.set_header(
        M.Label.new_int(
            M.Int.new_negative(
                M.BigNum.from_str("1"),
            ),
        ),
        M.CBORValue.new_int(
            M.Int.new_i32(6), //M.CurveType.Ed25519
        ),
    ); // crv (-1) set to Ed25519 (6)
    key.set_header(
        M.Label.new_int(
            M.Int.new_negative(
                M.BigNum.from_str("2"),
            ),
        ),
        M.CBORValue.new_bytes(priv.to_public().as_bytes()),
    ); // x (-2) set to public key

    return {
        signature: toHex(coseSign1.to_bytes()),
        key: toHex(key.to_bytes()),
    };
}

export const assetsToValue = async (assets: { unit: string, quantity: string }[]) => {
    const multiAsset = C.MultiAsset.new();
    const lovelace = assets.find((asset) => asset.unit === 'lovelace');
    const policies = [
        ...new Set(
            assets
                .filter((asset) => asset.unit !== 'lovelace')
                .map((asset) => asset.unit.slice(0, 56))
        ),
    ];
    policies.forEach((policy) => {
        const policyAssets = assets.filter(
            (asset) => asset.unit.slice(0, 56) === policy
        );
        const assetsValue = C.Assets.new();
        policyAssets.forEach((asset) => {
            assetsValue.insert(
                C.AssetName.new(Buffer.from(asset.unit.slice(56), 'hex')),
                C.BigNum.from_str(asset.quantity)
            );
        });
        multiAsset.insert(
            C.ScriptHash.from_bytes(Buffer.from(policy, 'hex')),
            assetsValue
        );
    });
    const value = C.Value.new(
        C.BigNum.from_str(lovelace ? lovelace.quantity : '0')
    );
    if (assets.length > 1 || !lovelace) value.set_multiasset(multiAsset);
    return value;
};

export const utxoFromJson = async (
    output: any,
    address: string
) => {
    return C.TransactionUnspentOutput.new(
        C.TransactionInput.new(
            C.TransactionHash.from_bytes(
                Buffer.from(output.tx_hash || output.txHash, 'hex')
            ),
            C.BigNum.from_str(
                (output.output_index ?? output.txId).toString()
            )
        ),
        C.TransactionOutput.new(
            C.Address.from_bytes(Buffer.from(address, 'hex')),
            await assetsToValue(output.amount)
        )
    );

};





// ----------------- CIP 30 -----------------

export class Web3AuthWalletAPI {
    paymentKey: PrivateKey
    stakeKey: PrivateKey
    network: Network
    blockfrostUrl: string
    blockfrostKey: string

    constructor(paymentKey: PrivateKey, stakeKey: PrivateKey, network: Network, blockfrostKey: string, blockfrostUrl: string) {
        this.paymentKey = paymentKey
        this.stakeKey = stakeKey
        this.network = network
        this.blockfrostUrl = blockfrostUrl
        this.blockfrostKey = blockfrostKey
    }
    async enable(): Promise<Web3AuthWalletAPI> {
        return this
    }

    async getNetworkId(): Promise<number> {
        if (this.network === 'Mainnet') {
            return 1
        } else {
            return 0
        }
        return 0
    }
    async getUtxos(): Promise<string[] | undefined> {
        const { address, stakeAddress } = getAddressesFromKeys(this.paymentKey, this.stakeKey, this.network)
        const addressHex = Buffer.from(C.Address.from_bech32(address).to_bytes()).toString('hex')

        const result = await getCardanoAddressUtxos(address, this.blockfrostUrl, this.blockfrostKey)

        const converted = await Promise.all(
            result.map(async (utxo: any) => {
                const coreUtxo = await utxoFromJson(utxo, addressHex)
                return Buffer.from(coreUtxo.to_bytes()).toString('hex')
            }
            )
        );
        return converted;
    }

    async getBalance(): Promise<string> {
        const { address, stakeAddress } = getAddressesFromKeys(this.paymentKey, this.stakeKey, this.network)
        const result = await getCardanoAddressInfo(address, this.blockfrostUrl, this.blockfrostKey)
        const value = await assetsToValue(result.amount);
        return Buffer.from(value.to_bytes()).toString('hex')
    }

    async getUsedAddresses(): Promise<string[]> {
        const { address } = getAddressesFromKeys(this.paymentKey, this.stakeKey, this.network)
        return [Buffer.from(C.Address.from_bech32(address).to_bytes()).toString('hex')];
    }
    async getUnusedAddresses(): Promise<string[]> {
        const { address } = getAddressesFromKeys(this.paymentKey, this.stakeKey, this.network)
        return [Buffer.from(C.Address.from_bech32(address).to_bytes()).toString('hex')];
    }
    async getRewardAddresses(): Promise<string[]> {
        const {  stakeAddress } = getAddressesFromKeys(this.paymentKey, this.stakeKey, this.network)
        return [Buffer.from(C.Address.from_bech32(stakeAddress).to_bytes()).toString('hex')];
    }
    async getChangeAddress(): Promise<string> {
        const { address } = getAddressesFromKeys(this.paymentKey, this.stakeKey, this.network)
        return Buffer.from(C.Address.from_bech32(address).to_bytes()).toString('hex');
    }

    async signTx(tx: string, partialSign: boolean): Promise<string> {
        
        const paymentKeyHash = Buffer.from(
            this.paymentKey.to_public().hash().to_bytes()
        ).toString('hex');
        const stakeKeyHash = Buffer.from(
            this.stakeKey.to_public().hash().to_bytes()
        ).toString('hex');

        const rawTx = C.Transaction.from_bytes(Buffer.from(tx, 'hex'));
        const txWitnessSet = C.TransactionWitnessSet.new();
        const vkeyWitnesses = C.Vkeywitnesses.new();
        const txHash = C.hash_transaction(rawTx.body());
        const utxos: TransactionUnspentOutput[] = (await this.getUtxos())?.map((utxo: string) => C.TransactionUnspentOutput.from_bytes(Buffer.from(utxo, 'hex'))) || []
        const keyHashes = getKeyHashes(this.paymentKey, this.stakeKey, rawTx, utxos)

        keyHashes.forEach((keyHash: any) => {

            let signingKey;
            if (keyHash === paymentKeyHash) signingKey = this.paymentKey;
            else if (keyHash === stakeKeyHash) signingKey = this.stakeKey;
            //else if (!partialSign) throw TxSignError.ProofGeneration;
            else return;
            const vkey = C.make_vkey_witness(txHash, signingKey!);
            vkeyWitnesses.add(vkey);
        });

        //stakeKey!.free();
        //stakeKey = null;
        //paymentKey!.free();
        //paymentKey = null;

        txWitnessSet.set_vkeys(vkeyWitnesses);
        return Buffer.from(txWitnessSet.to_bytes()).toString('hex');
    }

    async signData(address: string, payload: string): Promise<{ signature: string; key: string }> {
        if (address.startsWith('e0') || address.startsWith('e1')) {
            return signDataUtil(address, payload, this.stakeKey.to_bech32())//await lucid.wallet.signMessage(rewardAddress, message)
        } else {
            return signDataUtil(address, payload, this.paymentKey.to_bech32())//await lucid.wallet.signMessage(rewardAddress, message)
        }
    }

    async submitTx(tx: string): Promise<string> {
        const transaction = C.Transaction.from_bytes(Buffer.from(tx, 'hex'))
        const result = await fetch(
            `${this.blockfrostUrl}/tx/submit`,
            {
                headers: {
                    project_id: this.blockfrostKey as string,
                    "Content-Type": "application/cbor",
                },
                method: 'POST',
                body: transaction.to_bytes()

            },
        ).then((res) => res.json());

        if (result.error) {
            if (result.status_code === 400)
                throw { ...new Error("Failed to send the transaction"), message: result.message };
            else if (result.status_code === 500) throw new Error("Internal Error");
            else if (result.status_code === 429) throw new Error("Transaction refused. Try again later");
            else if (result.status_code === 425) throw new Error("Mempool is full. Try again later");
            else throw new Error("Invalid Request");
        }
        return result.toString('hex');
    }

    async getCollateral(): Promise<string[]> {
        return []
    };

    experimental = {
        getCollateral: this.getCollateral,
        on: () => { },
        off: () => { }
    }
}
