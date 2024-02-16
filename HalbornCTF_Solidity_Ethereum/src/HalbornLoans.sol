// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {HalbornToken} from "./HalbornToken.sol";
import {HalbornNFT} from "./HalbornNFT.sol";

import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import {Initializable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import {MulticallUpgradeable} from "./libraries/Multicall.sol";
import {IERC721ReceiverUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/token/ERC721/IERC721ReceiverUpgradeable.sol";

contract HalbornLoans is Initializable, UUPSUpgradeable, MulticallUpgradeable, IERC721ReceiverUpgradeable {
    
    // Here, a token and a NFT are declared
    HalbornToken public token;
    HalbornNFT public nft;

    // Is an immutable variable representing the price of the collateral.
    uint256 public immutable collateralPrice;

    // Here is a declaration of 3 mappings that are going to inclide a collection of collaterals
    // Accounts to identifiers and then identifiers to account
    mapping(address => uint256) public totalCollateral;
    mapping(address => uint256) public usedCollateral;
    mapping(uint256 => address) public idsCollateral;

    // In the constructor, the price is initialized
    constructor(uint256 collateralPrice_) {
        collateralPrice = collateralPrice_;
    }

    // In the initialize function the objects from the imports are initialized and the token and the
    // NFT are also initialized
    function initialize(address token_, address nft_) public initializer {
        __UUPSUpgradeable_init();
        __Multicall_init();

        token = HalbornToken(token_);
        nft = HalbornNFT(nft_);
    }

    // The next function allows us to deposit a NFT collateral
    function depositNFTCollateral(uint256 id) external {
        // First of all it is necessary that the owner of the NFT with ID "id" it is the one that
        // is executing the contract
        require(
            nft.ownerOf(id) == msg.sender,
            "Caller is not the owner of the NFT"
        );
        // If the previous condition is met, the NFT is transferred from the sender using the address 
        // of the current HalbornLoans certificate
        nft.safeTransferFrom(msg.sender, address(this), id);
        // The total collateral value of the account executing the action is increased with the collateral
        // price
        totalCollateral[msg.sender] += collateralPrice;
        // The account of the sender is included in the array registering the accounts for that NFT "id"
        idsCollateral[id] = msg.sender;
    }

    // This function allows us to withdraw a collateral, the oposite of the previous one
    function withdrawCollateral(uint256 id) external {
        // The condition is that the current collateral of the sender is greater or equal
        // than the current collateral price
        require(
            totalCollateral[msg.sender] - usedCollateral[msg.sender] >=
                collateralPrice,
            "Collateral unavailable"
        );
        // Also, the current ID should be linked to the actual sender
        require(idsCollateral[id] == msg.sender, "ID not deposited by caller");

        // If the conditions are ok, the collateral is transferred to the sender from the certificate 
        // HalbornLoans
        nft.safeTransferFrom(address(this), msg.sender, id);
        // We reduce the collateral of the sender with the collateral price
        totalCollateral[msg.sender] -= collateralPrice;
        // The information of the sender is deleted from the collaterals array
        delete idsCollateral[id];
    }

    using SafeMath for uint256;
    // The next function allows the sender to get a loan
    function getLoan(uint256 amount) external {
        // The only condition is that the collateral of the sender is less than the amount of loan 
        // requested
        require(
            totalCollateral[msg.sender] - usedCollateral[msg.sender] >=
                collateralPrice,
            "Collateral unavailable"
        );
        // Then the used collateral is increased with the amount
        usedCollateral[msg.sender] = usedCollateral[msg.sender].add(amount);
        // And a token is generated linking the user account to the amount requested in the loan
        token.mintToken(msg.sender, amount);
    }

    // This function allows the sender to return a loan
    function returnLoan(uint256 amount) external {
        // It is necessary that the used collateral is greater or equal than the amount
        require(usedCollateral[msg.sender] >= amount, "Not enough collateral");
        // Also we require that the balance of the token is greater or equal too
        require(token.balanceOf(msg.sender) >= amount, "Not enough token balance");
        // We update the collateral increasing it with the amount of the loan
        usedCollateral[msg.sender] += amount;
        // Finally we destroy the token with the sender and amount details
        token.burnToken(msg.sender, amount);
    }

    function _authorizeUpgrade(address) internal override {}

    function onERC721Received(
        address operator,
        address from,
        uint256 tokenId,
        bytes calldata data
    ) external override returns (bytes4) {
        // Check if the operator is the expected contract or address
        require(operator == address(this), "Unexpected operator");

        // Log the receipt of the ERC721 token
        emit ERC721Received(operator, from, tokenId, data);

        // Return the ERC721_RECEIVED selector as per ERC721 standard
        return IERC721ReceiverUpgradeable.onERC721Received.selector;
    }
    
    // Event to log the receipt of the ERC721 token
    event ERC721Received(
        address operator,
        address from,
        uint256 tokenId,
        bytes data
    );

}
