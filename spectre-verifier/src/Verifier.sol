// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "sp1-contracts/contracts/src/ISP1Verifier.sol";

struct RotationOutputStruct {
    uint32 slot;
    bytes32 commitment;
    bytes32 finalized_header_root;
    bytes32 next_commitment;
}

struct StepOutputStruct {
    uint32 slot;
    bytes32 commitment;
    bytes32 finalized_header_root;
}

/// @title Committee Verifier
/// @author Chainsafe Systems
/// @notice Verify a committee update
contract LightClientVerifier {
    /// @notice The address of the SP1 verifier contract.
    /// @dev This can either be a specific SP1Verifier for a specific version, or the
    ///      SP1VerifierGateway which can be used to verify proofs for any version of SP1.
    ///      For the list of supported verifiers on each chain, see:
    ///      https://github.com/succinctlabs/sp1-contracts/tree/main/contracts/deployments
    address public verifier;
    /// @notice The verification key for the committee program.
    bytes32 public committeeProgramVKey;
    /// @notice The verification key for the step program.
    bytes32 public stepProgramVKey;
    /// @notice The current finalized header root.
    bytes32 public finalizedHeaderRoot;
    // the current active committee
    bytes32 public activeCommitteeCommitment;
    // the current active slot
    uint32 public activeSlot;

    constructor(
        address _verifier,
        bytes32 _committeeProgramVKey,
        bytes32 _stepProgramVKey,
        bytes32 _finalizedHeaderRoot,
        bytes32 _activeCommitteeCommitment,
        uint32 _activeSlot
    ) {
        verifier = _verifier;
        committeeProgramVKey = _committeeProgramVKey;
        stepProgramVKey = _stepProgramVKey;
        finalizedHeaderRoot = _finalizedHeaderRoot;
        activeCommitteeCommitment = _activeCommitteeCommitment;
        activeSlot = _activeSlot;
    }

    /// @notice Verify the committee proof.
    /// @param _proofBytes The encoded proof.
    /// @param _publicValues The encoded public values.
    function verifyRotationProof(
        bytes calldata _publicValues,
        bytes calldata _proofBytes
    ) external {
        ISP1Verifier(verifier).verifyProof(
            committeeProgramVKey,
            _publicValues,
            _proofBytes
        );
        RotationOutputStruct memory publicValues = abi.decode(
            _publicValues,
            (RotationOutputStruct)
        );
        assert(activeCommitteeCommitment == publicValues.commitment);
        assert(publicValues.slot > activeSlot);
        activeSlot = publicValues.slot;
        activeCommitteeCommitment = publicValues.next_commitment;
        finalizedHeaderRoot = publicValues.finalized_header_root;
    }

    /// @notice Verify the step proof.
    /// @param _proofBytes The encoded proof.
    /// @param _publicValues The encoded public values.
    function verifyStepProof(
        bytes calldata _publicValues,
        bytes calldata _proofBytes
    ) external {
        ISP1Verifier(verifier).verifyProof(
            stepProgramVKey,
            _publicValues,
            _proofBytes
        );
        StepOutputStruct memory publicValues = abi.decode(
            _publicValues,
            (StepOutputStruct)
        );
        assert(activeCommitteeCommitment == publicValues.commitment);
        assert(publicValues.slot > activeSlot);
        finalizedHeaderRoot = publicValues.finalized_header_root;
    }
}
