// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {LightClientVerifier} from "../src/Verifier.sol";
import {SP1VerifierGateway} from "sp1-contracts/contracts/src/SP1VerifierGateway.sol";

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

contract RotationTest is Test {
    using stdJson for string;

    address verifier;
    LightClientVerifier public lc_verifier;

    function loadRotationFixture()
        public
        view
        returns (RotationProofFixture memory)
    {
        string memory root = vm.projectRoot();
        string memory path = string.concat(
            root,
            "/test/fixtures/rotation-groth16.json"
        );
        string memory json = vm.readFile(path);
        RotationProofFixture memory rotation_fixture;
        rotation_fixture.slot = abi.decode(json.parseRaw(".slot"), (uint32));
        rotation_fixture.root = abi.decode(json.parseRaw(".root"), (bytes32));
        rotation_fixture.commitment = abi.decode(
            json.parseRaw(".commitment"),
            (bytes32)
        );
        rotation_fixture.next_commitment = abi.decode(
            json.parseRaw(".nextCommitment"),
            (bytes32)
        );
        rotation_fixture.vkey = abi.decode(json.parseRaw(".vkey"), (bytes32));
        rotation_fixture.publicValues = abi.decode(
            json.parseRaw(".publicValues"),
            (bytes)
        );
        rotation_fixture.proof = abi.decode(json.parseRaw(".proof"), (bytes));
        return rotation_fixture;
    }

    function loadStepFixture() public view returns (StepProofFixture memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(
            root,
            "/test/fixtures/step-groth16.json"
        );
        string memory json = vm.readFile(path);
        StepProofFixture memory step_fixture;
        step_fixture.slot = abi.decode(json.parseRaw(".slot"), (uint32));
        step_fixture.root = abi.decode(json.parseRaw(".root"), (bytes32));
        step_fixture.commitment = abi.decode(
            json.parseRaw(".commitment"),
            (bytes32)
        );
        step_fixture.vkey = abi.decode(json.parseRaw(".vkey"), (bytes32));
        step_fixture.publicValues = abi.decode(
            json.parseRaw(".publicValues"),
            (bytes)
        );
        step_fixture.proof = abi.decode(json.parseRaw(".proof"), (bytes));
        return step_fixture;
    }

    function setUp() public {
        RotationProofFixture memory rotation_fixture = loadRotationFixture();
        StepProofFixture memory step_fixture = loadStepFixture();
        verifier = address(new SP1VerifierGateway(address(1)));
        lc_verifier = new LightClientVerifier(
            verifier,
            rotation_fixture.vkey, // either a deterministic build (nix, docker), or derived from ELF
            step_fixture.vkey,
            0x00,
            0xf670c64f8ed2c43554ef01e0a1b702b443e8709cd58c9c5330f1421dc9fcfef6, // see fixture payload
            6815743 // rotation fixture payload - 1,
        );
    }

    function test_ValidRotationProof() public {
        RotationProofFixture memory rotation_fixture = loadRotationFixture();
        vm.mockCall(
            verifier,
            abi.encodeWithSelector(SP1VerifierGateway.verifyProof.selector),
            abi.encode(true)
        );
        (
            uint32 slot,
            bytes32 finalized_header_root,
            bytes32 commitment,
            bytes32 next_commitment
        ) = lc_verifier.verifyRotationProof(
                rotation_fixture.publicValues,
                rotation_fixture.proof
            );
    }

    function test_ValidStepProof() public {
        StepProofFixture memory step_fixture = loadStepFixture();
        vm.mockCall(
            verifier,
            abi.encodeWithSelector(SP1VerifierGateway.verifyProof.selector),
            abi.encode(true)
        );
        (
            uint32 slot,
            bytes32 finalized_header_root,
            bytes32 commitment
        ) = lc_verifier.verifyStepProof(
                step_fixture.publicValues,
                step_fixture.proof
            );
    }

    function testFail_InvalidRotationProof() external {
        RotationProofFixture memory rotation_fixture = loadRotationFixture();
        bytes memory fakeProof = new bytes(rotation_fixture.proof.length);
        lc_verifier.verifyRotationProof(
            rotation_fixture.publicValues,
            fakeProof
        );
    }
}
