# block
An elementary account based POS blockchain written in Rust

## description
block is a POS blockchain utilizing many of the same crypto primitives as Bitcoin, but instead of being UTXO based it is account based. block's consensus model randomly chooses a validator with chances proportional to the amount of funds that each validator has staked to propose a new block every 5 minutes.

## documentation
Full documentation on the block protocol can be found here (coming soon).

## usage
#### *building*
For a production build of the block node software run:\
`cargo build --release`

#### *running*
Navigate to the `block` executable usually in ./target/release/ (if built in release mode)

`./block <config-file-path>`

By default the config file will be named "block.conf" and will be located in the same directory as the block executable. If there is no config file located at the path you provide, or if you do not provide a path and it is not located in the directory the block executable is located in "block.conf" will be created for you with default values. If you do provide a path to a config file and the file does not exist it will be created for you with the name you specified and in the directory you specified. Be sure any directory you specifiy already exists or the block node will fail to run.

## configuration file
The configuration file is written in toml and contains various sections with options related to that section title. If the configuration file is incomplete or incorrect the block node will fail to run. The current default configuration can be found below.
```
[wallet]
wallet_file = "block.wallet"
compressed_public_key = true
wallet_file_version = 1
```
### wallet
The wallet section is for configuration options related to the wallet module\
`wallet_file` - The path to the desired wallet file. Much like the config file if it doesn't exist it will be placed in the location that is specified.\
`compressed_public_key` - Defines if the wallet should derive its address from a compressed public key or not.\
`wallet_file_version` - The version of the wallet_file, it is unlikely for a new wallet file version to be implemented in the future.

## features
### wallet
The wallet module can create a private key, and save it to disk for subsequent restorations. The wallet module also maintains a nonce value used in transactions to prevent replay attacks. This number is to be incremented with each confirmed transaction from a specific address or the network will not accept the transaction. From the private key the wallet has generated it can generate a block address in a similar fashion as bitcoin version 1 addresses are generated. A block address is a 39 byte [base58](https://en.bitcoin.it/wiki/Base58Check_encoding) encoded string starting with the string `BLoCK` (block addresses must preserve capitalization). An example address is such: `BLoCK1RBq8BTN8kHLiL2wnwU79qEq4ujpGUiePh`. The wallet module also has the ability to convert its private key into [WIF](https://en.bitcoin.it/wiki/Wallet_import_format) (wallet import format) to store the key and restore from a private key in WIF format. The corresponding WIF private key to the example address earlier is such: `L5hEXzWhsgjuzf17ZT2zpsE54QC9poU3ZriT2nBsN7482qzKGYHq`. WIF private keys can start with either `K` or `L` if they correspond to an address generated from a compressed private key, or `5` if they correspond to an address generated from an uncompressed public key. Lastly, the wallet module has the ability to create and sign transactions on behalf of the user that are intended to be broadcast to the block network.

### transactions
Participants in the block network can broadcast transcations that transfer value from on participant to the other. A transaction consists of 7 distinct fields: version, amount, fee, recipient, sender, signature, and nonce. The block network has 4 distinct transaction types. They are as follows: A to B, coinbase, validator enable, and validator revoke.
#### *A to B*
A to B is a typical value transfer transcation. This transcation takes funds from participant A and transfers them to participant B. Participant A may also pay a transaction fee to entice validators to include their transaction in the block they are proposing. Participant A must have sufficient funds in their account, the total of the transaction amount plus the transaction fees, to send funds to participant B. Particpant A sends funds to participant B using participant B's block address derived from participant B's private key. Participant A will input the transaction amount, transaction fee, and intended recipient into their wallet software to construct a transaction. Their wallet software will broadcast the transcation to the network. Network participants will validate the transaction, confirm participant A has enough funds and participant A is attempting to spend funds they control, a validator will include their transaction in a block and participant A's funds will be available to participant B to utilize.
#### *coinbase*
A coinbase transaction is a specific type of transaction that mints new funds into the block network. It can only be initiated by the validator that was chosen to propose the current block. The validator will insert this transaction within the list of transactions for that specific block. This type of transaction shall have a fee of 0. The transaction amount that is to be minted and distributed shall follow the same reward schedule as bitcoin. 50 tokens will initially be distributed per block, and that token amount will halve every 210,000 blocks. At an average block proposal rate of 5 minutes the reward rate will halve about every 2 years. The recipient of this transaction can be any block address, including other particpants that is not the validator of that block.
#### *validator enable*
The validator enable transaction's purpose is to notify the network that a participant desires to become a validator within the network. To become a validator a participant will have to create a validator enable transaction with some minimum amount of funds to stake. Staking is the process of locking up funds that cannot be spent for the duration that they are staked and participants can earn rewards for validating blocks. During the bootstrapping phase of the block network validators will not be required to stake any funds to provide initial liquidity into the network, but after the bootstrapping phase is complete validators will be required to stake some minimum amount of funds. 
#### *validator revoke*
The validator revoke transaction is to signify to the network that a current validator no longer wishes to be a validator of the network. This transaction will allow the validator to reclaim the funds that they have staked, but it will no longer allow them to be a prosposer of new blocks or earn rewards for proposing new blocks.

### verification_engine
The verification engine module is responsible for verifying transactions and blocks. The verification engine can verify single transactions or blocks at the time. It also has the ability to take a stream of blocks with some initial starting chain state and determine if all of the blocks and transactions within them are valid. This feature is useful for initial block sync, and later block syncs if a node goes offline for some time.
