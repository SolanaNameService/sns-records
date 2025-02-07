import { PublicKey, SystemProgram } from "@solana/web3.js";
import {
  allocateAndPostRecordInstruction,
  allocateRecordInstruction,
  deleteRecordInstruction,
  editRecordInstruction,
  unverifyRoaInstruction,
  validateEthereumSignatureInstruction,
  validateSolanaSignatureInstruction,
  writeRoaInstruction,
} from "./raw_instructions";
import { Validation } from "./state";
import { Buffer } from "buffer";

/**
 * Mainnet program ID
 */
export const SNS_RECORDS_ID = new PublicKey(
  "HP3D4D1ZCmohQGFVms2SS4LCANgJyksBf5s1F77FuFjZ"
);


/**
 * Devnet program ID
 */

export const SNS_RECORD_ID_DEVNET = new PublicKey("Ga872GkshNeNMDag7m1Bn54dN3NiHksfqnN2pH6A1H9F")

/**
 * Central State
 */

export const [CENTRAL_STATE_SNS_RECORDS] = PublicKey.findProgramAddressSync(
  [SNS_RECORDS_ID.toBuffer()],
  SNS_RECORDS_ID
);

export const [CENTRAL_STATE_SNS_RECORDS_DEVNET] = PublicKey.findProgramAddressSync(
  [SNS_RECORD_ID_DEVNET.toBuffer()],
  SNS_RECORD_ID_DEVNET
);

/**
 * This function can be used as a js binding example.
 * @param feePayer The fee payer of the transaction
 * @param programId The program ID
 * @returns
 */
export const allocateAndPostRecord = (
  feePayer: PublicKey,
  recordKey: PublicKey,
  domainKey: PublicKey,
  domainOwner: PublicKey,
  nameProgramId: PublicKey,
  record: string,
  content: Buffer,
  programId: PublicKey
) => {
  const ix = new allocateAndPostRecordInstruction({
    record,
    content: Array.from(content),
  }).getInstruction(
    programId,
    SystemProgram.programId,
    nameProgramId,
    feePayer,
    recordKey,
    domainKey,
    domainOwner,
    CENTRAL_STATE_SNS_RECORDS
  );
  return ix;
};

export const allocateRecord = (
  feePayer: PublicKey,
  recordKey: PublicKey,
  domainKey: PublicKey,
  domainOwner: PublicKey,
  nameProgramId: PublicKey,
  record: string,
  contentLength: number,
  programId: PublicKey
) => {
  const ix = new allocateRecordInstruction({
    contentLength,
    record,
  }).getInstruction(
    programId,
    SystemProgram.programId,
    nameProgramId,
    feePayer,
    recordKey,
    domainKey,
    domainOwner,
    CENTRAL_STATE_SNS_RECORDS
  );
  return ix;
};

export const deleteRecord = (
  feePayer: PublicKey,
  domainKey: PublicKey,
  domainOwner: PublicKey,
  recordKey: PublicKey,
  nameProgramId: PublicKey,
  programId: PublicKey
) => {
  const ix = new deleteRecordInstruction().getInstruction(
    programId,
    SystemProgram.programId,
    nameProgramId,
    feePayer,
    recordKey,
    domainKey,
    domainOwner,
    CENTRAL_STATE_SNS_RECORDS
  );
  return ix;
};

export const editRecord = (
  feePayer: PublicKey,
  recordKey: PublicKey,
  domainKey: PublicKey,
  domainOwner: PublicKey,
  nameProgramId: PublicKey,
  record: string,
  content: Buffer,
  programId: PublicKey
) => {
  const ix = new editRecordInstruction({
    record,
    content: Array.from(content),
  }).getInstruction(
    programId,
    SystemProgram.programId,
    nameProgramId,
    feePayer,
    recordKey,
    domainKey,
    domainOwner,
    CENTRAL_STATE_SNS_RECORDS
  );
  return ix;
};

export const validateEthSignature = (
  feePayer: PublicKey,
  recordKey: PublicKey,
  domainKey: PublicKey,
  domainOwner: PublicKey,
  nameProgramId: PublicKey,
  validation: Validation,
  signature: Buffer,
  expectedPubkey: Buffer,
  programId: PublicKey
) => {
  const ix = new validateEthereumSignatureInstruction({
    validation,
    signature: Array.from(signature),
    expectedPubkey: Array.from(expectedPubkey),
  }).getInstruction(
    programId,
    SystemProgram.programId,
    nameProgramId,
    feePayer,
    recordKey,
    domainKey,
    domainOwner,
    CENTRAL_STATE_SNS_RECORDS
  );
  return ix;
};

export const validateSolanaSignature = (
  feePayer: PublicKey,
  recordKey: PublicKey,
  domainKey: PublicKey,
  domainOwner: PublicKey,
  verifier: PublicKey,
  nameProgramId: PublicKey,
  staleness: boolean,
  programId: PublicKey
) => {
  const ix = new validateSolanaSignatureInstruction({
    staleness,
  }).getInstruction(
    programId,
    SystemProgram.programId,
    nameProgramId,
    feePayer,
    recordKey,
    domainKey,
    domainOwner,
    CENTRAL_STATE_SNS_RECORDS,
    verifier
  );
  return ix;
};

export const writeRoa = (
  feePayer: PublicKey,
  nameProgramId: PublicKey,
  recordKey: PublicKey,
  domainKey: PublicKey,
  domainOwner: PublicKey,
  roaId: PublicKey,
  programId: PublicKey
) => {
  const ix = new writeRoaInstruction({
    roaId: Array.from(roaId.toBuffer()),
  }).getInstruction(
    programId,
    SystemProgram.programId,
    nameProgramId,
    feePayer,
    recordKey,
    domainKey,
    domainOwner,
    CENTRAL_STATE_SNS_RECORDS
  );
  return ix;
};

export const unverifyRoa = (
  feePayer: PublicKey,
  nameProgramId: PublicKey,
  recordKey: PublicKey,
  domainKey: PublicKey,
  verifier: PublicKey,
  programId: PublicKey
) => {
  const ix = new unverifyRoaInstruction().getInstruction(
    programId,
    SystemProgram.programId,
    nameProgramId,
    feePayer,
    recordKey,
    domainKey,
    CENTRAL_STATE_SNS_RECORDS,
    verifier
  );
  return ix;
};
