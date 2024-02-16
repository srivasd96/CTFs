// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {ERC721Upgradeable} from "openzeppelin-contracts-upgradeable/contracts/token/ERC721/ERC721Upgradeable.sol";
import {Initializable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/access/OwnableUpgradeable.sol";
import {MerkleProofUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/utils/cryptography/MerkleProofUpgradeable.sol";
import {MulticallUpgradeable} from "./libraries/Multicall.sol";

contract HalbornNFT is
    Initializable,
    ERC721Upgradeable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    MulticallUpgradeable
{
    bytes32 public merkleRoot;

    uint256 public price;
    uint256 public idCounter;

    // The NFT variables are initialized
    function initialize(
        bytes32 merkleRoot_,
        uint256 price_
    ) external initializer {
        __ERC721_init("Halborn NFT", "HNFT");
        __UUPSUpgradeable_init();
        __Ownable_init();
        __Multicall_init();

        setMerkleRoot(merkleRoot_);
        setPrice(price_);
    }

    // This functions allows to set a price to the NFT
    function setPrice(uint256 price_) public onlyOwner {
        require(price_ != 0, "Price cannot be 0");
        price = price_;
    }

    // This function allows to set the merkle root (Merkle roots are used in cryptocurrency 
    // to make sure data blocks passed between peers on a peer-to-peer network are whole, 
    // undamaged, and unaltered)
    function setMerkleRoot(bytes32 merkleRoot_) public {
        merkleRoot = merkleRoot_;
    }

    // The next function is used to generate the token
    function mintAirdrops(uint256 id, bytes32[] calldata merkleProof) external {
        require(!_exists(id), "Token already minted");

        bytes32 node = keccak256(abi.encodePacked(msg.sender, id));
        bool isValidProof = MerkleProofUpgradeable.verifyCalldata(
            merkleProof,
            merkleRoot,
            node
        );
        require(isValidProof, "Invalid proof.");

        _safeMint(msg.sender, id, "");
    }

    // This function allows to buy the token with ETH
    function mintBuyWithETH() external payable {
        require(msg.value == price, "Invalid Price");

        unchecked {
            idCounter++;
        }

        _safeMint(msg.sender, idCounter, "");
    }

    // Here you can withdraw some amount of ETH (only the owner of the NFT)
    function withdrawETH(uint256 amount) external onlyOwner {
        payable(owner()).transfer(amount);
    }

    function _authorizeUpgrade(address) internal override {}
}
