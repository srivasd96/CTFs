// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {ERC20Upgradeable} from "openzeppelin-contracts-upgradeable/contracts/token/ERC20/ERC20Upgradeable.sol";
import {Initializable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import {OwnableUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/access/OwnableUpgradeable.sol";
import {MulticallUpgradeable} from "./libraries/Multicall.sol";

contract HalbornToken is
    Initializable,
    ERC20Upgradeable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    MulticallUpgradeable
{
    address public halbornLoans;

    // Modifier that checks if the sender account is from HalbornLoans contract
    modifier onlyLoans() {
        require(msg.sender == halbornLoans, "Caller is not HalbornLoans");
        _;
    }

    // Here all the imports are initialized
    function initialize() external initializer {
        __ERC20_init("HalbornToken", "HT");
        __UUPSUpgradeable_init();
        __Ownable_init();
        __Multicall_init();
    }

    // Here you can set some loan account
    function setLoans(address halbornLoans_) external onlyOwner {
        require(halbornLoans_ != address(0), "Zero Address");
        halbornLoans = halbornLoans_;
    }
    // In this function a token related to some account an for some amount is minted
    function mintToken(address account, uint256 amount) external onlyLoans {
        _mint(account, amount);
    }
    // In this function a token related to some account and for some amount is burned
    function burnToken(address account, uint256 amount) external onlyLoans {
        _burn(account, amount);
    }

    function _authorizeUpgrade(address) internal override {}
}
