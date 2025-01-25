// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
import {stdJson} from "forge-std/StdJson.sol";

struct RotationProofFixture {
    uint32 slot;
    bytes32 root;
    bytes32 commitment;
    bytes32 next_commitment;
    bytes32 vkey;
    bytes publicValues;
    bytes proof;
}

struct StepProofFixture {
    uint32 slot;
    bytes32 root;
    bytes32 commitment;
    bytes32 vkey;
    bytes publicValues;
    bytes proof;
}

library FixtureLoader {
    using stdJson for string;

    function parseRotationFixture(
        string memory json
    ) internal pure returns (RotationProofFixture memory) {
        RotationProofFixture memory fixture;
        fixture.slot = abi.decode(json.parseRaw(".slot"), (uint32));
        fixture.root = abi.decode(json.parseRaw(".root"), (bytes32));
        fixture.commitment = abi.decode(
            json.parseRaw(".commitment"),
            (bytes32)
        );
        fixture.next_commitment = abi.decode(
            json.parseRaw(".nextCommitment"),
            (bytes32)
        );
        fixture.vkey = abi.decode(json.parseRaw(".vkey"), (bytes32));
        fixture.publicValues = abi.decode(
            json.parseRaw(".publicValues"),
            (bytes)
        );
        fixture.proof = abi.decode(json.parseRaw(".proof"), (bytes));
        return fixture;
    }

    function parseStepFixture(
        string memory json
    ) internal pure returns (StepProofFixture memory) {
        StepProofFixture memory fixture;
        fixture.slot = abi.decode(json.parseRaw(".slot"), (uint32));
        fixture.root = abi.decode(json.parseRaw(".root"), (bytes32));
        fixture.commitment = abi.decode(
            json.parseRaw(".commitment"),
            (bytes32)
        );
        fixture.vkey = abi.decode(json.parseRaw(".vkey"), (bytes32));
        fixture.publicValues = abi.decode(
            json.parseRaw(".publicValues"),
            (bytes)
        );
        fixture.proof = abi.decode(json.parseRaw(".proof"), (bytes));
        return fixture;
    }
}
