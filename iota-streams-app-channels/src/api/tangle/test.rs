#![allow(non_snake_case)]
use crate::api::tangle::{
    Address,
    Author,
    Subscriber,
};
use iota_streams_app::{
    message::HasLink,
    transport::tangle::PAYLOAD_BYTES,
};
use iota_streams_core::{
    prelude::{
        string::ToString,
        Rc,
    },
    println,
    try_or,
    Result,
    ensure,
    LOCATION_LOG,
    Errors::*,
};

use core::cell::RefCell;

use super::*;

pub fn example<T: Transport>(transport: T) -> Result<()>
{
    let encoding = "utf-8";
    let multi_branching = false;
    let transport = Rc::new(RefCell::new(transport));

    let mut author = Author::new(
        "AUTHOR9SEED",
        encoding,
        PAYLOAD_BYTES,
        multi_branching,
        transport.clone(),
    );

    let mut subscriberA = Subscriber::new("SUBSCRIBERA9SEED", encoding, PAYLOAD_BYTES, transport.clone());

    let mut subscriberB = Subscriber::new("SUBSCRIBERB9SEED", encoding, PAYLOAD_BYTES, transport.clone());

    let public_payload = Bytes("PUBLICPAYLOAD".as_bytes().to_vec());
    let masked_payload = Bytes("MASKEDPAYLOAD".as_bytes().to_vec());

    println!("announce");
    let (announcement_address, announcement_tag) = {
        let msg = &author.send_announce()?;
        println!("  {}", msg);
        (msg.appinst.to_string(), msg.msgid.to_string())
    };
    let announcement_link = Address::from_str(&announcement_address, &announcement_tag).unwrap();

    {
        subscriberA.receive_announcement(&announcement_link)?;
        ensure!(
            author.channel_address() == subscriberA.channel_address(),
            "bad channel address"
        );
        subscriberB.receive_announcement(&announcement_link)?;
        ensure!(
            subscriberA.channel_address() == subscriberB.channel_address(),
            "bad channel address"
        );
        ensure!(
            subscriberA
                .channel_address()
                .map_or(false, |appinst| appinst == announcement_link.base()),
            "bad announcement address"
        );
    }

    println!("\nsign packet");
    let signed_packet_link = {
        let (msg, _) = author.send_signed_packet(&announcement_link, &public_payload, &masked_payload)?;
        println!("  {}", msg);
        msg
    };
    println!("  at {}", signed_packet_link.rel());

    {
        let (_pk, unwrapped_public, unwrapped_masked) = subscriberA.receive_signed_packet(&signed_packet_link)?;
        try_or!(
            public_payload == unwrapped_public,
            PublicPayloadMismatch(public_payload.to_string(), unwrapped_public.to_string())
        )?;
        try_or!(
            masked_payload == unwrapped_masked,
            MaskedPayloadMismatch(masked_payload.to_string(), unwrapped_masked.to_string())
        )?;
    }

    println!("\nsubscribe");
    let subscribeB_link = {
        let msg = subscriberB.send_subscribe(&announcement_link)?;
        println!("  {}", msg);
        msg
    };

    {
        author.receive_subscribe(&subscribeB_link)?;
    }

    println!("\nshare keyload for everyone");
    let keyload_link = {
        let (msg, _) = author.send_keyload_for_everyone(&announcement_link)?;
        println!("  {}", msg);
        msg
    };

    {
        let resultA = subscriberA.receive_keyload(&keyload_link);
        let unwrapped = resultA.is_ok() && !resultA.unwrap();
        try_or!(unwrapped, SubscriberAccessMismatch("A".to_string()))?;
        let resultB = subscriberB.receive_keyload(&keyload_link)?;
        try_or!(resultB, MessageUnwrapFailure("B".to_string()))?;
    }

    println!("\ntag packet");
    let tagged_packet_link = {
        let (msg, _) = author.send_tagged_packet(&keyload_link, &public_payload, &masked_payload)?;
        println!("  {}", msg);
        msg
    };

    {
        let resultA = subscriberA.receive_tagged_packet(&tagged_packet_link);
        ensure!(resultA.is_err(), "subscriberA failed to unwrap tagged packet");
        let (unwrapped_public, unwrapped_masked) = subscriberB.receive_tagged_packet(&tagged_packet_link)?;
        ensure!(public_payload == unwrapped_public, "bad unwrapped public payload");
        ensure!(masked_payload == unwrapped_masked, "bad unwrapped masked payload");
    }

    {
        subscriberB.receive_keyload(&keyload_link)?;
    }

    let subAdump = subscriberA.export("pwdSubA").unwrap();
    let _subscriberA2 = Subscriber::import(subAdump.as_ref(), "pwdSubA", transport.clone()).unwrap();

    let subBdump = subscriberB.export("pwdSubB").unwrap();
    let _subscriberB2 = Subscriber::import(subBdump.as_ref(), "pwdSubB", transport.clone()).unwrap();

    let authordump = author.export("pwdAuthor").unwrap();
    let _author2 = Author::import(authordump.as_ref(), "pwdAuthor", transport.clone()).unwrap();

    Ok(())
}

#[test]
fn run_basic_scenario() {
    let transport = crate::api::tangle::BucketTransport::new();
    assert!(dbg!(example(transport)).is_ok());
}
