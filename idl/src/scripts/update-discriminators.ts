import * as idl from "../target/idl/usdf_swap.json";

// Instruction discriminators matching the actual program
// These values come from api/src/instruction.rs InstructionType enum
const instructionValues: Record<string, number[]> = {
    initialize: [1],
    swap: [2],
    transfer: [3],
};

// Account discriminators matching the actual program
// These values come from api/src/state/mod.rs AccountType enum
const accountValues: Record<string, number[]> = {
    Pool: [1, 0, 0, 0, 0, 0, 0, 0],
};

function updateDiscriminators() {
    const instructions = (idl as any).instructions;
    for (const ix of instructions) {
        const val = instructionValues[ix.name];
        if (val === undefined) {
            throw new Error(`Instruction ${ix.name} not found in discriminator map`);
        }
        ix.discriminator = val;
    }

    const accounts = (idl as any).accounts;
    for (const acc of accounts) {
        const val = accountValues[acc.name];
        if (val === undefined) {
            throw new Error(`Account ${acc.name} not found in discriminator map`);
        }
        acc.discriminator = val;
    }

    return idl;
}

console.log(JSON.stringify(updateDiscriminators(), null, 2));
