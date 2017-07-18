use std::mem::replace;
use futures::*;
use futures::sink;
use tokio_io::{AsyncRead, AsyncWrite};
use minidom::Element;
use sasl::common::Credentials;
use sasl::common::scram::*;
use sasl::client::Mechanism;
use sasl::client::mechanisms::*;
use serialize::base64::{self, ToBase64, FromBase64};

use xmpp_codec::*;
use xmpp_stream::*;
use stream_start::*;

const NS_XMPP_SASL: &str = "urn:ietf:params:xml:ns:xmpp-sasl";

pub struct ClientAuth<S: AsyncWrite> {
    state: ClientAuthState<S>,
    mechanism: Box<Mechanism>,
}

enum ClientAuthState<S: AsyncWrite> {
    WaitSend(sink::Send<XMPPStream<S>>),
    WaitRecv(XMPPStream<S>),
    Start(StreamStart<S>),
    Invalid,
}

impl<S: AsyncWrite> ClientAuth<S> {
    pub fn new(stream: XMPPStream<S>, creds: Credentials) -> Result<Self, String> {
        let mechs: Vec<Box<Mechanism>> = vec![
            Box::new(Scram::<Sha256>::from_credentials(creds.clone()).unwrap()),
            Box::new(Scram::<Sha1>::from_credentials(creds.clone()).unwrap()),
            Box::new(Plain::from_credentials(creds).unwrap()),
            Box::new(Anonymous::new()),
        ];

        let mech_names: Vec<String> =
            match stream.stream_features.get_child("mechanisms", NS_XMPP_SASL) {
                None =>
                    return Err("No auth mechanisms".to_owned()),
                Some(mechs) =>
                    mechs.children()
                    .filter(|child| child.is("mechanism", NS_XMPP_SASL))
                    .map(|mech_el| mech_el.text())
                    .collect(),
            };
        println!("SASL mechanisms offered: {:?}", mech_names);

        for mut mech in mechs {
            let name = mech.name().to_owned();
            if mech_names.iter().any(|name1| *name1 == name) {
                println!("SASL mechanism selected: {:?}", name);
                let initial = try!(mech.initial());
                let mut this = ClientAuth {
                    state: ClientAuthState::Invalid,
                    mechanism: mech,
                };
                this.send(
                    stream,
                    "auth", &[("mechanism", &name)],
                    &initial
                );
                return Ok(this);
            }
        }

        Err("No supported SASL mechanism available".to_owned())
    }

    fn send(&mut self, stream: XMPPStream<S>, nonza_name: &str, attrs: &[(&str, &str)], content: &[u8]) {
        let nonza = Element::builder(nonza_name)
            .ns(NS_XMPP_SASL);
        let nonza = attrs.iter()
            .fold(nonza, |nonza, &(name, value)| nonza.attr(name, value))
            .append(content.to_base64(base64::STANDARD))
            .build();

        let send = stream.send(Packet::Stanza(nonza));

        self.state = ClientAuthState::WaitSend(send);
    }
}

impl<S: AsyncRead + AsyncWrite> Future for ClientAuth<S> {
    type Item = XMPPStream<S>;
    type Error = String;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let state = replace(&mut self.state, ClientAuthState::Invalid);

        match state {
            ClientAuthState::WaitSend(mut send) =>
                match send.poll() {
                    Ok(Async::Ready(stream)) => {
                        self.state = ClientAuthState::WaitRecv(stream);
                        self.poll()
                    },
                    Ok(Async::NotReady) => {
                        self.state = ClientAuthState::WaitSend(send);
                        Ok(Async::NotReady)
                    },
                    Err(e) =>
                        Err(format!("{}", e)),
                },
            ClientAuthState::WaitRecv(mut stream) =>
                match stream.poll() {
                    Ok(Async::Ready(Some(Packet::Stanza(ref stanza))))
                        if stanza.name() == "challenge"
                        && stanza.ns() == Some(NS_XMPP_SASL) =>
                    {
                        let content = try!(
                            stanza.text()
                                .from_base64()
                                .map_err(|e| format!("{}", e))
                        );
                        let response = try!(self.mechanism.response(&content));
                        self.send(stream, "response", &[], &response);
                        self.poll()
                    },
                    Ok(Async::Ready(Some(Packet::Stanza(ref stanza))))
                        if stanza.name() == "success"
                        && stanza.ns() == Some(NS_XMPP_SASL) =>
                    {
                        let start = stream.restart();
                        self.state = ClientAuthState::Start(start);
                        self.poll()
                    },
                    Ok(Async::Ready(Some(Packet::Stanza(ref stanza))))
                        if stanza.name() == "failure"
                        && stanza.ns() == Some(NS_XMPP_SASL) =>
                    {
                        let mut e = None;
                        for child in stanza.children() {
                            e = Some(child.name().clone());
                            break
                        }
                        let e = e.unwrap_or_else(|| "Authentication failure");
                        Err(e.to_owned())
                    },
                    Ok(Async::Ready(event)) => {
                        println!("ClientAuth ignore {:?}", event);
                        Ok(Async::NotReady)
                    },
                    Ok(_) => {
                        self.state = ClientAuthState::WaitRecv(stream);
                        Ok(Async::NotReady)
                    },
                    Err(e) =>
                        Err(format!("{}", e)),
                },
            ClientAuthState::Start(mut start) =>
                match start.poll() {
                    Ok(Async::Ready(stream)) =>
                        Ok(Async::Ready(stream)),
                    Ok(Async::NotReady) => {
                        self.state = ClientAuthState::Start(start);
                        Ok(Async::NotReady)
                    },
                    Err(e) =>
                        Err(format!("{}", e)),
                },
            ClientAuthState::Invalid =>
                unreachable!(),
        }
    }
}
