use std::collections::BTreeSet;
use ic_agent::{Agent, agent, AgentError};
use candid::{decode_one, encode_args, encode_one};
use ic_agent::identity::{BasicIdentity, Secp256k1Identity};
use ic_agent::Identity;
use openssl::ec::{EcGroup, EcKey};
use openssl::nid::Nid;
use openssl::pkey::Private;
use ring::signature::Ed25519KeyPair;
use sha2::Digest;
use sha2::Sha256;
use std::fs;
use std::io::{Read, Write};
use cap_common::transaction::Event;
use cap_common::{TokenContractId, TransactionId};
use serde::{Serialize, Deserialize};
use ic_types::Principal;
use std::time::Duration;
use garcon::Delay;

const RETRY_PAUSE: Duration = Duration::from_millis(200);
const MAX_RETRY_PAUSE: Duration = Duration::from_secs(1);

pub fn waiter_with_exponential_backoff() -> Delay {
    Delay::builder()
        .exponential_backoff_capped(RETRY_PAUSE, 1.4, MAX_RETRY_PAUSE)
        .build()
}


#[derive(Deserialize, Serialize)]
pub struct CanisterList {
    pub(crate) data: Vec<Principal>,
    pub(crate) hash: [u8; 32],
}

#[derive(Deserialize, Serialize)]
pub struct Data {
    pub bucket: Vec<Event>,
    pub buckets: Vec<(TransactionId, Principal)>,
    pub next_canisters: CanisterList,
    pub users: BTreeSet<Principal>,
    pub cap_id: Principal,
    pub contract: TokenContractId,
    pub writers: BTreeSet<TokenContractId>,
    pub allow_migration: bool,
}

/// A key that can be converted to a Identity.
#[derive(Clone)]
pub enum PrivateKey {
    Ed25519Pkcs8(Vec<u8>),
    Secp256k1(EcKey<Private>),
}

impl PrivateKey {
    /// Try to load a pem file.
    pub fn from_pem_file<P: AsRef<std::path::Path>>(file_path: P) -> anyhow::Result<Self> {
        let reader = fs::File::open(file_path)?;
        let pem = reader
            .bytes()
            .collect::<Result<Vec<u8>, std::io::Error>>()?;

        if let Ok(private_key) = EcKey::private_key_from_pem_callback(&pem, |mut key| {
            panic!("Not supported.")
        }) {
            return Ok(Self::Secp256k1(private_key));
        }

        let pkcs8 = pem::parse(&pem)?.contents;
        // Validation step.
        Ed25519KeyPair::from_pkcs8(pkcs8.as_slice())?;

        Ok(Self::Ed25519Pkcs8(pkcs8))
    }

    pub fn generate() -> Self {
        let group = EcGroup::from_curve_name(Nid::SECP256K1).expect("Cannot create EcGroup.");
        let private_key = EcKey::generate(&group).unwrap();
        Self::Secp256k1(private_key)
    }

    /// Convert to an identity.
    pub fn into_identity(self) -> Box<dyn Identity> {
        match self {
            PrivateKey::Ed25519Pkcs8(pkcs8) => {
                let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_slice()).unwrap();
                Box::new(BasicIdentity::from_key_pair(key_pair))
            }
            PrivateKey::Secp256k1(private_key) => {
                Box::new(Secp256k1Identity::from_private_key(private_key))
            }
        }
    }
}

const PER_MESSAGE: usize = 1000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let id = std::env::args().nth(1).unwrap();
    let url = String::from("https://ic0.app");
    let identity = PrivateKey::from_pem_file(id)?.into_identity();
    let id = Principal::from_text("v6dvh-qawes-jzboy-lai6t-bzqrc-f5hym-clbrc-tdmht-x3msr-uzqgi-4qe").unwrap();
    assert_eq!(id, identity.sender().unwrap());

    let agent = Agent::builder()
        .with_transport(
            agent::http_transport::ReqwestHttpReplicaV2Transport::create(&url)
                .expect("Failed to create Transport for Agent"),
        )
        .with_boxed_identity(identity)
        .build().expect("Could not build the agent.");
    let canister_id = Principal::from_text("whq4n-xiaaa-aaaam-qaazq-cai").unwrap();

    let data = include_bytes!("./wicp.bin");
    let mut deserializer = serde_cbor::Deserializer::from_slice(data.as_slice());
    let value: Data = Deserialize::deserialize(&mut deserializer).unwrap();
    let events = value.bucket;
    assert_eq!(events.len(), 276092);

    let bytes = agent.query(&canister_id, "old_data_size").with_arg(encode_one(Vec::<Event>::new())).call().await.unwrap();
    let from = decode_one::<u64>(bytes.as_slice()).unwrap() as usize;
    let events = events.into_iter().skip(from).collect::<Vec<_>>();

    let mut i = 0;
    loop {
        let start = i * PER_MESSAGE;
        let end = (start + PER_MESSAGE).min(events.len());
        if start > events.len() {
            println!("Finished!");
            break;
        }

        let data = &events[start..end];
        let arg = encode_one(data).unwrap();
        match agent.update(&canister_id, "write_data_restore").with_arg(arg).call_and_wait(waiter_with_exponential_backoff()).await {
            Ok(_) => {
                println!("sent ({}..{}) successfully.", start, end);
                i += 1;
            }
            Err(e) => {
                println!("failed to send: {:?}", e);
            }
        }
    }

    Ok(())
}
