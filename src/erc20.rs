
use core::marker::PhantomData;

/// Import the Stylus SDK along with alloy primitive types for use in our program.
use stylus_sdk::{
    alloy_primitives::{U256, Address, B256, U160}, prelude::*,
    alloy_sol_types::sol,
    evm, block, crypto, msg, contract
};

sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}

pub trait UniswapV2ERC20Params {
	const NAME: &'static str = "Stylus Uniswap V2";
    const SYMBOL: &'static str = "STYLUS-UNI-V2";
    const DECIMALS: u8 = 18;
}

sol_storage! {
    pub struct UniswapV2ERC20<T> {
        uint256 total_supply;
        mapping(address => uint256) balances;
        mapping(address => mapping(address => uint256)) allowances;
        mapping(address => uint256) nonces;
        PhantomData<T> phantom;
    }
}

#[public]
impl <T: UniswapV2ERC20Params> UniswapV2ERC20<T> {
    pub fn name(&self) -> Result<String, Vec<u8>> {
        Ok(T::NAME.to_string())
    }
    pub fn symbol(&self) -> Result<String, Vec<u8>> {
        Ok(T::SYMBOL.to_string())
    }
    pub fn decimals(&self) -> Result<u8, Vec<u8>> {
        Ok(T::DECIMALS)
    }
    pub fn totalSupply(&self) -> Result<U256, Vec<u8>> {
        Ok(self.total_supply.get())
    }
    pub fn balanceOf(&self, address: Address) -> Result<U256, Vec<u8>> {
        Ok(self.balances.get(address))
    }
    pub fn allowance(&self, owner: Address, spender: Address) -> Result<U256, Vec<u8>> {
        Ok(self.allowances.getter(owner).get(spender))
    }

    pub fn approve(&mut self, spender: Address, value: U256) -> Result<bool, Vec<u8>> {
        self._approve(msg::sender(), spender, value);
        Ok(true)
    }

    pub fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Vec<u8>> {
        self._transfer(msg::sender(), to, value)?;
        Ok(true)
    }

    pub fn transferFrom(&mut self, from: Address, to: Address, value: U256) -> Result<bool, Vec<u8>> {
        let mut from_allowance = self.allowances.setter(from);
        let mut allowance = from_allowance.setter(msg::sender());
        let old_allowance = allowance.get();
        if old_allowance < value {
            return Err("Insufficient allowance".to_string().into_bytes());
        }
        allowance.set(old_allowance - value);
        self._transfer(from, to, value)?;
        Ok(true)
    }
}

impl<T:UniswapV2ERC20Params> UniswapV2ERC20<T> {
    pub fn _mint(&mut self, to: Address, value: U256) {
        let mut balance = self.balances.setter(to);
        let new_balance = balance.get() + value;
        balance.set(new_balance);
        self.total_supply.set(self.total_supply.get() + value);
        evm::log(Transfer {
            from: Address::ZERO,
            to,
            value,
        });
    }
    pub fn _burn(&mut self, from: Address, value: U256) {
        let mut balance = self.balances.setter(from);
        let new_balance = balance.get() - value;
        balance.set(new_balance);
        self.total_supply.set(self.total_supply.get() - value);
        evm::log(Transfer {
            from,
            to: Address::ZERO,
            value,
        });
    }
    pub fn _approve(&mut self, owner: Address, spender: Address, value: U256) {
        let mut allowance = self.allowances.setter(owner);
        allowance.setter(spender).set(value);
        evm::log(Approval { owner, spender, value });
    }
    pub fn _transfer(&mut self, from: Address, to: Address, value: U256) -> Result<(), Vec<u8>> {
        let mut from_balance = self.balances.setter(from);
        let old_from_balance = from_balance.get();
        if old_from_balance < value {
            return Err("Insufficient balance".to_string().into_bytes());
        }
        from_balance.set(old_from_balance - value);
        let mut to_balance = self.balances.setter(to);
        let new_to_balance = to_balance.get() + value;
        to_balance.set(new_to_balance);
        evm::log(Transfer { from, to, value });
        Ok(())
    }
}