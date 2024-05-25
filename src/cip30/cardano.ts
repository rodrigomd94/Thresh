import {  BlockfrostUTXO, CardanoAddressInfo } from "../types/cardano";


export const getCardanoAddressInfo = async (address: string, blockfrostUrl:string, blockfrostKey: string): Promise<CardanoAddressInfo>=> {
    const result = await fetch(
        `${blockfrostUrl}/addresses/${address}`,
        {
            headers: {
                project_id: blockfrostKey,
                "Content-Type": "application/json",
            },
        },
    ).then((res) => res.json());

    if (result.error) {
        if (result.status_code === 400) throw new Error("Invalid Request");
        else if (result.status_code === 500) throw new Error("Internal Error");
        // else address not found because it's a new address
        else {
            return {
                address: address,
                amount: [],
                stake_address: "",
                type: "byron",
                script: false
             }
        }
        // else return Buffer.from(C.Value.new(C.BigNum.from_str('0')).to_bytes()).toString('hex');
    }
    return result
}


export const getCardanoAddressUtxos = async (address: string, blockfrostUrl: string, blockfrostKey:string) => {
    let result: BlockfrostUTXO[] = [];
    let page = 1// paginate && paginate.page ? paginate.page + 1 : 1;
    const limit = ''//paginate && paginate.limit ? `&count=${paginate.limit}` : '';
    while (true) {
        let pageResult = await fetch(
            `${blockfrostUrl}/addresses/${address}/utxos?page=${page}${limit}`,
            {
                headers: {
                    project_id: blockfrostKey,
                    "Content-Type": "application/json",
                },
            },
        ).then((res) => res.json());

        if (pageResult.error) {
            if (pageResult.status_code === 400) throw new Error("Invalid Request");
            else if (pageResult.status_code === 500) throw new Error("Internal Error");
            else {
                pageResult = [];
            }
        }
        result = result.concat(pageResult);
        if (pageResult.length <= 0 /* || paginate */) break;
        page++;
    }
    return result
}



