import { expect, use } from "chai";
import chaiAsPromised from "chai-as-promised";
import { encodeAddress } from "@polkadot/keyring";
import BN from "bn.js";
import Shiden34_factory from "../types/constructors/shiden34";
import Shiden34 from "../types/contracts/shiden34";

import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { ReturnNumber } from "@727-ventures/typechain-types";
import { Id, IdBuilder } from "../types/types-arguments/shiden34";

use(chaiAsPromised);

const MAX_SUPPLY = 888;
const BASE_URI = "ipfs://tokenUriPrefix/";
const COLLECTION_METADATA = "ipfs://collectionMetadata/data.json";
const TOKEN_URI_1 = "ipfs://tokenUriPrefix/1.json";
const TOKEN_URI_5 = "ipfs://tokenUriPrefix/5.json";
const ONE = new BN(0);
const PRICE_PER_MINT = ONE;

// Create a new instance of contract
const wsProvider = new WsProvider("ws://127.0.0.1:9944");
// Create a keyring instance
const keyring = new Keyring({ type: "sr25519" });

describe("Minting psp34 tokens", () => {
  let shiden34Factory: Shiden34_factory;
  let api: ApiPromise;
  let deployer: KeyringPair;
  let bob: KeyringPair;
  let projectAccount: KeyringPair;
  let contract: Shiden34;

  const gasLimit = 18750000000;
  const ZERO_ADDRESS = encodeAddress(
    "0x0000000000000000000000000000000000000000000000000000000000000000"
  );
  let gasRequired: bigint;

  async function setup(): Promise<void> {
    api = await ApiPromise.create({ provider: wsProvider });
    deployer = keyring.addFromUri("//Alice");
    bob = keyring.addFromUri("//Bob");
    projectAccount = keyring.addFromUri("//Charlie");
    shiden34Factory = new Shiden34_factory(api, deployer);
    contract = new Shiden34(
      (
        await shiden34Factory.new(
          ["Shiden34"],
          ["SH34"],
          [BASE_URI],
          PRICE_PER_MINT,
        )
      ).address,
      deployer,
      api
    );
  }

  it("Create collection works", async () => {
    await setup();
    const queryList = await contract.query;
    expect(
      (await contract.query.totalSupply()).value.rawNumber.toNumber()
    ).to.equal(0);
    expect((await contract.query.owner()).value).to.equal(deployer.address);
    expect((await contract.query.maxSupply()).value.rawNumber.toNumber()).to.equal(0);
    expect((await contract.query.price()).value.rawNumber.toString()).to.equal(
      PRICE_PER_MINT.toString()
    );

    // expect((await contract.query.getAttribute({u128: collectionId}, ["baseUri"])).value).to.equal(BASE_URI);
    // expect((await contract.query.getAttribute(collectionId, ["baseUri"])).value).to.equal(BASE_URI);
  });

  it("Use mintNext works", async () => {
    await setup();
    const tokenId: Id = IdBuilder.U64(1);

    expect(
      (await contract.query.totalSupply()).value.rawNumber.toNumber()
    ).to.equal(0);

    // mint
    const { gasRequired } = await contract.withSigner(bob).query.mintNext();
    let mintResult = await contract
      .withSigner(bob)
      .tx.mintNext({ value: PRICE_PER_MINT, gasLimit: gasRequired * 2n });

    // verify minting results. The totalSupply value is BN
    expect(
      (await contract.query.totalSupply()).value.rawNumber.toNumber()
    ).to.equal(1);
    expect((await contract.query.balanceOf(bob.address)).value).to.equal(1);
    expect((await contract.query.balanceOf(bob.address)).value).to.equal(1);

    expect((await contract.query.ownerOf(tokenId)).value).to.equal(
      bob.address
    );
    emit(mintResult, "Transfer", {
      from: null,
      to: bob.address,
      id: tokenId,
    });
  });

  it("Token transfer works", async () => {
    await setup();

    // Bob mints
    let { gasRequired } = await contract.withSigner(bob).query.mintNext();
    let mintResult = await contract
      .withSigner(bob)
      .tx.mintNext({ value: PRICE_PER_MINT, gasLimit: gasRequired * 2n });

    const firstTokenId = IdBuilder.U64(
      (await contract.query.tokenByIndex(0)).value.ok.u64
    );

    emit(mintResult, "Transfer", {
      from: null,
      to: bob.address,
      id: firstTokenId,
    });

    // Bob transfers token to Deployer
    const transferGas = (
      await contract
        .withSigner(bob)
        .query.transfer(deployer.address, firstTokenId, [])
    ).gasRequired;
    let transferResult = await contract
      .withSigner(bob)
      .tx.transfer(deployer.address, firstTokenId, [], {
        gasLimit: transferGas,
      });

    // Verify transfer
    expect((await contract.query.ownerOf(firstTokenId)).value).to.equal(
      deployer.address
    );
    expect((await contract.query.balanceOf(bob.address)).value).to.equal(0);
    emit(transferResult, "Transfer", {
      from: bob.address,
      to: deployer.address,
      id: firstTokenId,
    });
  });

  it("Token approval works", async () => {
    await setup();

    // Bob mints
    let { gasRequired } = await contract.withSigner(bob).query.mintNext();
    await contract
      .withSigner(bob)
      .tx.mintNext({ value: PRICE_PER_MINT, gasLimit: gasRequired * 2n });

    const firstTokenId = IdBuilder.U64(
      (await contract.query.tokenByIndex(0)).value.ok.u64
    );

    // Bob approves deployer to be operator of the token
    const approveGas = (
      await contract
        .withSigner(bob)
        .query.approve(deployer.address, firstTokenId, true)
    ).gasRequired;
    let approveResult = await contract
      .withSigner(bob)
      .tx.approve(deployer.address, firstTokenId, true, {
        gasLimit: approveGas,
      });

    // Verify that Bob is still the owner and allowance is set
    expect((await contract.query.ownerOf(firstTokenId)).value).to.equal(
      bob.address
    );
    expect(
      (
        await contract.query.allowance(
          bob.address,
          deployer.address,
          firstTokenId
        )
      ).value
    ).to.equal(true);
    emit(approveResult, "Approval", {
      from: bob.address,
      to: deployer.address,
      id: firstTokenId,
      approved: true,
    });
  });

  it("Cannot mint after mint_end true", async () => {
    await setup();

    expect(
      (await contract.query.totalSupply()).value.rawNumber.toNumber()
    ).to.equal(0);

    await contract
      .withSigner(bob)
      .tx.setMintEnd(true);

    const mintEnd = contract.query.getMintEnd();

    expect((await mintEnd).value).to.equal(true);

    // mint
    const result = await contract.withSigner(bob).query.mintNext();

    expect(result.value.err.custom).to.equal("0x4d696e74456e64")
  });

  it("Cannot mint twice", async() => {
    await setup();
    expect(
      (await contract.query.totalSupply()).value.rawNumber.toNumber()
    ).to.equal(0);

    let alreadyMinted = await contract.query.getIsAccountMinted(bob.address);

    expect(alreadyMinted.value).to.equal(false);
    // mint
    const { gasRequired } = await contract.withSigner(bob).query.mintNext();
    await contract
      .withSigner(bob)
      .tx.mintNext({ value: PRICE_PER_MINT, gasLimit: gasRequired * 2n });


    alreadyMinted = await contract.query.getIsAccountMinted(bob.address);
    expect(alreadyMinted.value).to.equal(true);

    const result = await contract.withSigner(bob).query.mintNext();

    expect(result.value.err.custom).to.equal("0x43616e6e6f744d696e744d6f72655468616e4f6e6365")

  })
});

// Helper function to parse Events
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function emit(result: { events?: any }, name: string, args: any): void {
  const event = result.events.find(
    (event: { name: string }) => event.name === name
  );
  for (const key of Object.keys(event.args)) {
    if (event.args[key] instanceof ReturnNumber) {
      event.args[key] = event.args[key].toNumber();
    }
  }
  expect(event).eql({ name, args });
}

// Helper function to convert error code to string
function hex2a(psp34CustomError: any): string {
  var hex = psp34CustomError.toString(); //force conversion
  var str = "";
  for (var i = 0; i < hex.length; i += 2)
    str += String.fromCharCode(parseInt(hex.substr(i, 2), 16));
  return str.substring(1);
}