// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
import {Test, console} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {LightClientVerifier} from "../src/Verifier.sol";
import {SP1Verifier} from "sp1-contracts/contracts/src/v4.0.0-rc.3/SP1VerifierGroth16.sol";
import {FixtureLoader, RotationProofFixture, StepProofFixture} from "../src/Fixture.sol";

contract RotationTest is Test {
    using stdJson for string;

    address verifier;
    LightClientVerifier public lc_verifier;

    function setUp() public {
        string memory root = vm.projectRoot();
        string memory path = string.concat(
            root,
            "/test/fixtures/rotation-groth16.json"
        );
        string memory json = vm.readFile(path);
        RotationProofFixture memory rotation_fixture = FixtureLoader
            .parseRotationFixture(json);
        path = string.concat(root, "/test/fixtures/step-groth16.json");
        json = vm.readFile(path);
        StepProofFixture memory step_fixture = FixtureLoader.parseStepFixture(
            json
        );
        // Deploy the actual SP1Verifier contract
        verifier = address(new SP1Verifier());
        // Deploy the LightClientVerifier using the deployed SP1Verifier
        lc_verifier = new LightClientVerifier(
            verifier,
            rotation_fixture.vkey, // either a deterministic build (nix, docker), or derived from ELF
            step_fixture.vkey,
            0x00,
            0x0fbe43bcd87569860753ec08e275400ec8d307259e618ca8319b03e714f06523, // see fixture payload
            0x403a49cd58a653a01aa3e2031a5ec52cd8199d555b5ed67643aab6a42e40bcef,
            6823935 // attested slot - 1 for testing!
        );
    }

    function test_ValidRotationProof() public {
        string memory root = vm.projectRoot();
        string memory path = string.concat(
            root,
            "/test/fixtures/rotation-groth16.json"
        );
        string memory json = vm.readFile(path);
        RotationProofFixture memory rotation_fixture = FixtureLoader
            .parseRotationFixture(json);

        // Actual call to SP1Verifier
        lc_verifier.verifyRotationProof(
            rotation_fixture.publicValues,
            rotation_fixture.proof
        );
    }

    function test_ValidStepProof() public {
        string memory root = vm.projectRoot();
        string memory path = string.concat(
            root,
            "/test/fixtures/step-groth16.json"
        );
        string memory json = vm.readFile(path);
        StepProofFixture memory step_fixture = FixtureLoader.parseStepFixture(
            json
        );
        lc_verifier.verifyStepProof(
            step_fixture.publicValues,
            step_fixture.proof
        );
    }

    function testFail_InvalidRotationProof() external {
        string memory root = vm.projectRoot();
        string memory path = string.concat(
            root,
            "/test/fixtures/rotation-groth16.json"
        );
        string memory json = vm.readFile(path);
        RotationProofFixture memory rotation_fixture = FixtureLoader
            .parseRotationFixture(json);

        // Generate a fake proof to simulate invalid proof behavior
        bytes memory fakeProof = new bytes(rotation_fixture.proof.length);
        lc_verifier.verifyRotationProof(
            rotation_fixture.publicValues,
            fakeProof
        );
    }
}
