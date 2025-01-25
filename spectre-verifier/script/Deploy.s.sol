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
            0x0000000000000000000000000000000000000000000000000000000000000000,
            0x0fbe43bcd87569860753ec08e275400ec8d307259e618ca8319b03e714f06523,
            6823935
        );
        vm.stopBroadcast();
        // Log deployed contract address
        console.log("LightClientVerifier deployed to:", address(lc_verifier));
    }
}

/*
    Groth16Verifier 4.0.0 0xa27A057CAb1a4798c6242F6eE5b2416B7Cd45E5D
    LightClientVerifier deployed to: 0x6B7d0B0681C1f7353b616DaB7c45FDF37e252d4C
    cast abi-encode "constructor(address,bytes32,bytes32,bytes32,bytes32,uint32)" 0xa27A057CAb1a4798c6242F6eE5b2416B7Cd45E5D 0x00c979baa333567e3d7be2d7212cc6e9303c2ef57d5456d59d1b44bad5e210e2 0x009e70c4c7d07976ec221e7798e946c13e7d9135f078069aae30aeb55f866349 0x0000000000000000000000000000000000000000000000000000000000000000 0x0fbe43bcd87569860753ec08e275400ec8d307259e618ca8319b03e714f06523 6823935
*/
