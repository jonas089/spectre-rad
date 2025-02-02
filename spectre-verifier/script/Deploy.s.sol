// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
import "forge-std/Script.sol";
import {LightClientVerifier} from "../src/Verifier.sol";
import {FixtureLoader, RotationProofFixture, StepProofFixture} from "../src/Fixture.sol";
import "forge-std/console.sol";

contract DeployMyContract is Script {
    address verifier;
    LightClientVerifier public lc_verifier;

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);
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
        // Deploy contracts
        verifier = address(0xa27A057CAb1a4798c6242F6eE5b2416B7Cd45E5D);
        lc_verifier = new LightClientVerifier(
            verifier,
            rotation_fixture.vkey,
            step_fixture.vkey,
            0x451aa7a54cd6a7df4f68b69a21c0ee765296b82a2a5251ed4650a1c4863e607f,
            0x665a1457f3e9d4b0df4c552243b4fefdfcd1416da02ce43f8961bd43317da35e,
            0xf4a0c8a8df3e7ab1b5c09a4c317b0e872107fb150eaebaa771c70ca94fb86f98,
            6889472
        );
        vm.stopBroadcast();
        // Log deployed contract address
        console.log("LightClientVerifier deployed to:", address(lc_verifier));
    }
}

/*
    Groth16Verifier 4.0.0 0xa27A057CAb1a4798c6242F6eE5b2416B7Cd45E5D
    LightClientVerifier deployed to: 0xb55B42591446Fe7aaCd142D1676a6Fe4ee86182E
    cast abi-encode "constructor(address,bytes32,bytes32,bytes32,bytes32,bytes32,uint32)" 0xa27A057CAb1a4798c6242F6eE5b2416B7Cd45E5D 0x00c3f1e7c7dcc0d2006ff06665a4f6e389a1f5b2c89054874ce5488c8903a0dd 0x00546574af9f634da33f100eb3f375461ee7e6d3c2c0fdc74a9ba91eedc42f65 0x451aa7a54cd6a7df4f68b69a21c0ee765296b82a2a5251ed4650a1c4863e607f 0x665a1457f3e9d4b0df4c552243b4fefdfcd1416da02ce43f8961bd43317da35e 0xf4a0c8a8df3e7ab1b5c09a4c317b0e872107fb150eaebaa771c70ca94fb86f98 6889472
*/
