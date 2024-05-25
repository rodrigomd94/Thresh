//import {encode, decode} from 'hex-encode-decode'
export const toHex = (bytes: Uint8Array): string => {
    return Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("");
    //return decode(bytes);
}


export const fromHex = (hex: string): Uint8Array => {
    return new Uint8Array(hex.match(/.{1,2}/g)!.map((byte) => parseInt(byte, 16)));
    //return encode(hex);
}