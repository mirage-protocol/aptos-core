/// By deploying this module, the deployer will be creating a new managed fungible asset with the hardcoded
/// maximum supply, name, symbol, and decimals. The address of the asset can be obtained via get_asset().
/// The deployer will also become the initial admin and can mint/burn/freeze/unfreeze accounts.
/// The admin can transfer the asset via object::transfer() at any point to set a new admin.
module fungible_asset::managed_fungible_asset {
    use aptos_framework::fungible_asset::{Self, MintRef, TransferRef, BurnRef, ExtractedAsset, Metadata};
    use aptos_framework::object::{Self, Object};
    use aptos_framework::primary_wallet;
    use std::error;
    use std::signer;
    use std::string::utf8;

    /// Only fungible asset metadata owner can make changes.
    const ENOT_OWNER: u64 = 1;

    const ASSET_SYMBOL: vector<u8> = b"APT";

    #[resource_group_member(group = aptos_framework::object::ObjectGroup)]
    /// Hold refs to control the minting, transfer and burning of fungible assets.
    struct ManagedFungibleAsset has key {
        mint_ref: MintRef,
        transfer_ref: TransferRef,
        burn_ref: BurnRef,
    }

    /// Initialize metadata object and store the refs.
    fun init_module(admin: &signer) {
        let constructor_ref = &object::create_named_object(admin, ASSET_SYMBOL);
        primary_wallet::create_primary_wallet_enabled_fungible_asset(
            constructor_ref,
            0, /* maximum_supply. 0 means no maximum */
            utf8(b"Aptos Token"), /* name */
            utf8(ASSET_SYMBOL), /* symbol */
            8, /* decimals */
        );

        // Create mint/burn/transfer refs to allow creator to manage the fungible asset.
        let mint_ref = fungible_asset::generate_mint_ref(constructor_ref);
        let burn_ref = fungible_asset::generate_burn_ref(constructor_ref);
        let transfer_ref = fungible_asset::generate_transfer_ref(constructor_ref);
        let metadata_object_signer = object::generate_signer(constructor_ref);
        move_to(
            &metadata_object_signer,
            ManagedFungibleAsset { mint_ref, transfer_ref, burn_ref }
        )
    }

    #[view]
    /// Return the address of the managed fungible asset that's created when this module is deployed.
    public fun get_asset(): Object<Metadata> {
        let asset_address = object::create_object_address(&@fungible_asset, ASSET_SYMBOL);
        object::address_to_object<Metadata>(asset_address)
    }

    /// Mint as the owner of metadata object.
    public entry fun mint(admin: &signer, amount: u64, to: address) acquires ManagedFungibleAsset {
        let asset = get_asset();
        let managed_fungible_asset = authorized_borrow_refs(admin, asset);
        let to_wallet = primary_wallet::ensure_primary_wallet_exists(to, asset);
        let fa = fungible_asset::mint(&managed_fungible_asset.mint_ref, amount);
        fungible_asset::deposit_with_ref(&managed_fungible_asset.transfer_ref, to_wallet, fa);
    }

    /// Transfer as the owner of metadata object ignoring `allow_ungated_transfer` field.
    public entry fun transfer(admin: &signer, from: address, to: address, amount: u64) acquires ManagedFungibleAsset {
        let asset = get_asset();
        let transfer_ref = &authorized_borrow_refs(admin, asset).transfer_ref;
        let from_wallet = primary_wallet::ensure_primary_wallet_exists(from, asset);
        let to_wallet = primary_wallet::ensure_primary_wallet_exists(to, asset);
        fungible_asset::transfer_with_ref(transfer_ref, from_wallet, to_wallet, amount);
    }

    /// Burn fungible assets as the owner of metadata object.
    public entry fun burn(admin: &signer, from: address, amount: u64) acquires ManagedFungibleAsset {
        let asset = get_asset();
        let burn_ref = &authorized_borrow_refs(admin, asset).burn_ref;
        let from_wallet = primary_wallet::ensure_primary_wallet_exists(from, asset);
        fungible_asset::burn(burn_ref, from_wallet, amount);
    }

    /// Freeze an account so it cannot transfer or receive fungible assets.
    public entry fun freeze_account(admin: &signer, account: address) acquires ManagedFungibleAsset {
        let asset = get_asset();
        let transfer_ref = &authorized_borrow_refs(admin, asset).transfer_ref;
        let wallet = primary_wallet::ensure_primary_wallet_exists(account, asset);
        fungible_asset::set_ungated_transfer(transfer_ref, wallet, false);
    }

    /// Unfreeze an account so it can transfer or receive fungible assets.
    public entry fun unfreeze_account(admin: &signer, account: address) acquires ManagedFungibleAsset {
        let asset = get_asset();
        let transfer_ref = &authorized_borrow_refs(admin, asset).transfer_ref;
        let wallet = primary_wallet::ensure_primary_wallet_exists(account, asset);
        fungible_asset::set_ungated_transfer(transfer_ref, wallet, true);
    }

    /// Withdraw as the owner of metadata object ignoring `allow_ungated_transfer` field.
    public fun withdraw(admin: &signer, amount: u64, from: address): ExtractedAsset acquires ManagedFungibleAsset {
        let asset = get_asset();
        let transfer_ref = &authorized_borrow_refs(admin, asset).transfer_ref;
        let from_wallet = primary_wallet::ensure_primary_wallet_exists(from, asset);
        fungible_asset::withdraw_with_ref(transfer_ref, from_wallet, amount)
    }

    /// Deposit as the owner of metadata object ignoring `allow_ungated_transfer` field.
    public fun deposit(admin: &signer, to: address, fa: ExtractedAsset) acquires ManagedFungibleAsset {
        let asset = get_asset();
        let transfer_ref = &authorized_borrow_refs(admin, asset).transfer_ref;
        let to_wallet = primary_wallet::ensure_primary_wallet_exists(to, asset);
        fungible_asset::deposit_with_ref(transfer_ref, to_wallet, fa);
    }

    /// Borrow the immutable reference of the refs of `metadata`.
    /// This validates that the signer is the metadata object's owner.
    inline fun authorized_borrow_refs(
        owner: &signer,
        asset: Object<Metadata>,
    ): &ManagedFungibleAsset acquires ManagedFungibleAsset {
        assert!(object::is_owner(asset, signer::address_of(owner)), error::permission_denied(ENOT_OWNER));
        borrow_global<ManagedFungibleAsset>(object::object_address(&asset))
    }

    #[test(creator = @0xcafe)]
    fun test_basic_flow(
        creator: &signer,
    ) acquires ManagedFungibleAsset {
        init_module(creator);
        let creator_address = signer::address_of(creator);
        let aaron_address = @0xface;

        mint(creator, 100, creator_address);
        let asset = get_asset();
        assert!(primary_wallet::balance(creator_address, asset) == 100, 4);
        freeze_account(creator, creator_address);
        assert!(!primary_wallet::ungated_transfer_allowed(creator_address, asset), 5);
        transfer(creator, creator_address, aaron_address, 10);
        assert!(primary_wallet::balance(aaron_address, asset) == 10, 6);

        unfreeze_account(creator, creator_address);
        assert!(primary_wallet::ungated_transfer_allowed(creator_address, asset), 7);
        burn(creator, creator_address, 90);
    }

    #[test(creator = @0xcafe, aaron = @0xface)]
    #[expected_failure(abort_code = 0x50001, location = Self)]
    fun test_permission_denied(
        creator: &signer,
        aaron: &signer
    ) acquires ManagedFungibleAsset {
        init_module(creator);
        let creator_address = signer::address_of(creator);
        mint(aaron, 100, creator_address);
    }
}
