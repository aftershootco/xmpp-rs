// Copyright (c) 2017 Maxime “pep” Buquet <pep+code@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;
use std::convert::TryInto;
use std::str::FromStr;

use minidom::{Element, IntoElements, IntoAttributeValue, ElementEmitter};

use jid::Jid;

use error::Error;

use ns;

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    /// Status: 100
    NonAnonymousRoom,

    /// Status: 101
    AffiliationChange,

    /// Status: 102
    ConfigShowsUnavailableMembers,

    /// Status: 103
    ConfigHidesUnavailableMembers,

    /// Status: 104
    ConfigNonPrivacyRelated,

    /// Status: 110
    SelfPresence,

    /// Status: 170
    ConfigRoomLoggingEnabled,

    /// Status: 171
    ConfigRoomLoggingDisabled,

    /// Status: 172
    ConfigRoomNonAnonymous,

    /// Status: 173
    ConfigRoomSemiAnonymous,

    /// Status: 201
    RoomHasBeenCreated,

    /// Status: 210
    AssignedNick,

    /// Status: 301
    Banned,

    /// Status: 303
    NewNick,

    /// Status: 307
    Kicked,

    /// Status: 321
    RemovalFromRoom,

    /// Status: 322
    ConfigMembersOnly,

    /// Status: 332
    ServiceShutdown,
}

impl TryFrom<Element> for Status {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Status, Error> {
        if !elem.is("status", ns::MUC_USER) {
            return Err(Error::ParseError("This is not a status element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in status element."));
        }
        for (attr, _) in elem.attrs() {
            if attr != "code" {
                return Err(Error::ParseError("Unknown attribute in status element."));
            }
        }
        let code = get_attr!(elem, "code", required);

        Ok(match code {
             100 => Status::NonAnonymousRoom,
             101 => Status::AffiliationChange,
             102 => Status::ConfigShowsUnavailableMembers,
             103 => Status::ConfigHidesUnavailableMembers,
             104 => Status::ConfigNonPrivacyRelated,
             110 => Status::SelfPresence,
             170 => Status::ConfigRoomLoggingEnabled,
             171 => Status::ConfigRoomLoggingDisabled,
             172 => Status::ConfigRoomNonAnonymous,
             173 => Status::ConfigRoomSemiAnonymous,
             201 => Status::RoomHasBeenCreated,
             210 => Status::AssignedNick,
             301 => Status::Banned,
             303 => Status::NewNick,
             307 => Status::Kicked,
             321 => Status::RemovalFromRoom,
             322 => Status::ConfigMembersOnly,
             332 => Status::ServiceShutdown,
             _ => return Err(Error::ParseError("Invalid status code.")),
        })
    }
}

impl Into<Element> for Status {
    fn into(self) -> Element {
        Element::builder("status")
                .ns(ns::MUC_USER)
                .attr("code", match self {
                     Status::NonAnonymousRoom => 100,
                     Status::AffiliationChange => 101,
                     Status::ConfigShowsUnavailableMembers => 102,
                     Status::ConfigHidesUnavailableMembers => 103,
                     Status::ConfigNonPrivacyRelated => 104,
                     Status::SelfPresence => 110,
                     Status::ConfigRoomLoggingEnabled => 170,
                     Status::ConfigRoomLoggingDisabled => 171,
                     Status::ConfigRoomNonAnonymous => 172,
                     Status::ConfigRoomSemiAnonymous => 173,
                     Status::RoomHasBeenCreated => 201,
                     Status::AssignedNick => 210,
                     Status::Banned => 301,
                     Status::NewNick => 303,
                     Status::Kicked => 307,
                     Status::RemovalFromRoom => 321,
                     Status::ConfigMembersOnly => 322,
                     Status::ServiceShutdown => 332,
                })
                .build()
    }
}

impl IntoElements for Status {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_child(self.into());
    }
}

/// Optional <actor/> element used in <item/> elements inside presence stanzas of type
/// "unavailable" that are sent to users who are kick or banned, as well as within IQs for tracking
/// purposes. -- CHANGELOG  0.17 (2002-10-23)
/// Possesses a 'jid' and a 'nick' attribute, so that an action can be attributed either to a real
/// JID or to a roomnick. -- CHANGELOG  1.25 (2012-02-08)
#[derive(Debug, Clone, PartialEq)]
pub enum Actor {
    Jid(Jid),
    Nick(String),
}

impl TryFrom<Element> for Actor {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Actor, Error> {
        if !elem.is("actor", ns::MUC_USER) {
            return Err(Error::ParseError("This is not a actor element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in actor element."));
        }
        for (attr, _) in elem.attrs() {
            if attr != "jid" && attr != "nick" {
                return Err(Error::ParseError("Unknown attribute in actor element."));
            }
        }
        let jid: Option<Jid> = get_attr!(elem, "jid", optional);
        let nick = get_attr!(elem, "nick", optional);

        match (jid, nick) {
            (Some(_), Some(_))
          | (None, None) =>
                return Err(Error::ParseError("Either 'jid' or 'nick' attribute is required.")),
            (Some(jid), _) => Ok(Actor::Jid(jid)),
            (_, Some(nick)) => Ok(Actor::Nick(nick)),
        }
    }
}

impl Into<Element> for Actor {
    fn into(self) -> Element {
        let elem = Element::builder("actor").ns(ns::MUC_USER);

        (match self {
            Actor::Jid(jid) => elem.attr("jid", String::from(jid)),
            Actor::Nick(nick) => elem.attr("nick", nick),
        }).build()
    }
}

impl IntoElements for Actor {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_child(self.into());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Continue {
    thread: Option<String>,
}

impl TryFrom<Element> for Continue {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Continue, Error> {
        if !elem.is("continue", ns::MUC_USER) {
            return Err(Error::ParseError("This is not a continue element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in continue element."));
        }
        for (attr, _) in elem.attrs() {
            if attr != "thread" {
                return Err(Error::ParseError("Unknown attribute in continue element."));
            }
        }
        Ok(Continue { thread: get_attr!(elem, "thread", optional) })
    }
}

impl Into<Element> for Continue {
    fn into(self) -> Element {
        Element::builder("continue")
                .ns(ns::MUC_USER)
                .attr("thread", self.thread)
                .build()
    }
}

impl IntoElements for Continue {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_child(self.into());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reason(String);

impl TryFrom<Element> for Reason {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Reason, Error> {
        if !elem.is("reason", ns::MUC_USER) {
            return Err(Error::ParseError("This is not a reason element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in reason element."));
        }
        for _ in elem.attrs() {
            return Err(Error::ParseError("Unknown attribute in reason element."));
        }
        Ok(Reason(elem.text()))
    }
}

impl Into<Element> for Reason {
    fn into(self) -> Element {
        Element::builder("reason")
                .ns(ns::MUC_USER)
                .append(self.0)
                .build()
    }
}

impl IntoElements for Reason {
    fn into_elements(self, emitter: &mut ElementEmitter) {
        emitter.append_child(self.into());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Affiliation {
    Owner,
    Admin,
    Member,
    Outcast,
    None,
}

impl FromStr for Affiliation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Affiliation, Error> {
        Ok(match s {
            "owner" => Affiliation::Owner,
            "admin" => Affiliation::Admin,
            "member" => Affiliation::Member,
            "outcast" => Affiliation::Outcast,
            "none" => Affiliation::None,

            _ => return Err(Error::ParseError("Unknown affiliation.")),
        })
    }
}

impl IntoAttributeValue for Affiliation {
    fn into_attribute_value(self) -> Option<String> {
        Some(String::from(match self {
            Affiliation::Owner => "owner",
            Affiliation::Admin => "admin",
            Affiliation::Member => "member",
            Affiliation::Outcast => "outcast",
            Affiliation::None => "none",
        }))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Moderator,
    Participant,
    Visitor,
    None,
}

impl FromStr for Role {
    type Err = Error;

    fn from_str(s: &str) -> Result<Role, Error> {
        Ok(match s {
            "moderator" => Role::Moderator,
            "participant" => Role::Participant,
            "visitor" => Role::Visitor,
            "none" => Role::None,

            _ => return Err(Error::ParseError("Unknown role.")),
        })
    }
}

impl IntoAttributeValue for Role {
    fn into_attribute_value(self) -> Option<String> {
        Some(String::from(match self {
            Role::Moderator => "moderator",
            Role::Participant => "participant",
            Role::Visitor => "visitor",
            Role::None => "none",
        }))
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub affiliation: Affiliation,
    pub jid: Option<Jid>,
    pub nick: Option<String>,
    pub role: Role,
    pub actor: Option<Actor>,
    pub continue_: Option<Continue>,
    pub reason: Option<Reason>,
}

impl TryFrom<Element> for Item {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Item, Error> {
        if !elem.is("item", ns::MUC_USER) {
            return Err(Error::ParseError("This is not a item element."));
        }
        let mut actor: Option<Actor> = None;
        let mut continue_: Option<Continue> = None;
        let mut reason: Option<Reason> = None;
        for child in elem.children() {
            if child.is("actor", ns::MUC_USER) {
                actor = Some(child.clone().try_into()?);
            } else if child.is("continue", ns::MUC_USER) {
                continue_ = Some(child.clone().try_into()?);
            } else if child.is("reason", ns::MUC_USER) {
                reason = Some(child.clone().try_into()?);
            } else {
                return Err(Error::ParseError("Unknown child in item element."));
            }
        }
        for (attr, _) in elem.attrs() {
            if attr != "affiliation" && attr != "jid" &&
               attr != "nick" && attr != "role" {
                return Err(Error::ParseError("Unknown attribute in item element."));
            }
        }

        let affiliation: Affiliation = get_attr!(elem, "affiliation", required);
        let jid: Option<Jid> = get_attr!(elem, "jid", optional);
        let nick: Option<String> = get_attr!(elem, "nick", optional);
        let role: Role = get_attr!(elem, "role", required);

        Ok(Item{
            affiliation: affiliation,
            jid: jid,
            nick: nick,
            role: role,
            actor: actor,
            continue_: continue_,
            reason: reason,
        })
    }
}

impl Into<Element> for Item {
    fn into(self) -> Element {
        Element::builder("item")
                .ns(ns::MUC_USER)
                .attr("affiliation", self.affiliation)
                .attr("jid", match self.jid {
                    Some(jid) => Some(String::from(jid)),
                    None => None,
                })
                .attr("nick", self.nick)
                .attr("role", self.role)
                .append(self.actor)
                .append(self.continue_)
                .append(self.reason)
                .build()
    }
}

#[derive(Debug, Clone)]
pub struct MucUser {
    pub status: Vec<Status>,
    pub items: Vec<Item>,
}

impl TryFrom<Element> for MucUser {
    type Error = Error;

    fn try_from(elem: Element) -> Result<MucUser, Error> {
        if !elem.is("x", ns::MUC_USER) {
            return Err(Error::ParseError("This is not an x element."));
        }
        let mut status = vec!();
        let mut items = vec!();
        for child in elem.children() {
            if child.is("status", ns::MUC_USER) {
                status.push(Status::try_from(child.clone())?);
            } else if child.is("item", ns::MUC_USER) {
                items.push(Item::try_from(child.clone())?);
            } else {
                return Err(Error::ParseError("Unknown child in x element."));
            }
        }
        for _ in elem.attrs() {
            return Err(Error::ParseError("Unknown attribute in x element."));
        }
        Ok(MucUser {
            status,
            items,
        })
    }
}

impl Into<Element> for MucUser {
    fn into(self) -> Element {
        Element::builder("x")
                .ns(ns::MUC_USER)
                .append(self.status)
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_simple() {
        let elem: Element = "
            <x xmlns='http://jabber.org/protocol/muc#user'/>
        ".parse().unwrap();
        MucUser::try_from(elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "
            <x xmlns='http://jabber.org/protocol/muc#user'>
                <coucou/>
            </x>
        ".parse().unwrap();
        let error = MucUser::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in x element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "
            <x xmlns='http://jabber.org/protocol/muc#user'/>
        ".parse().unwrap();
        let muc = MucUser { status: vec!(), items: vec!() };
        let elem2 = muc.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "
            <x xmlns='http://jabber.org/protocol/muc#user' coucou=''/>
        ".parse().unwrap();
        let error = MucUser::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in x element.");
    }

    #[test]
    fn test_status_simple() {
        let elem: Element = "
            <status xmlns='http://jabber.org/protocol/muc#user' code='110'/>
        ".parse().unwrap();
        Status::try_from(elem).unwrap();
    }

    #[test]
    fn test_status_invalid() {
        let elem: Element = "
            <status xmlns='http://jabber.org/protocol/muc#user'/>
        ".parse().unwrap();
        let error = Status::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'code' missing.");
    }

    #[test]
    fn test_status_invalid_child() {
        let elem: Element = "
            <status xmlns='http://jabber.org/protocol/muc#user' code='110'>
                <foo/>
            </status>
        ".parse().unwrap();
        let error = Status::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in status element.");
    }

    #[test]
    fn test_status_simple_code() {
        let elem: Element = "
            <status xmlns='http://jabber.org/protocol/muc#user' code='307'/>
        ".parse().unwrap();
        let status = Status::try_from(elem).unwrap();
        assert_eq!(status, Status::Kicked);
    }

    #[test]
    fn test_status_invalid_code() {
        let elem: Element = "
            <status xmlns='http://jabber.org/protocol/muc#user' code='666'/>
        ".parse().unwrap();
        let error = Status::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid status code.");
    }

    #[test]
    fn test_status_invalid_code2() {
        let elem: Element = "
            <status xmlns='http://jabber.org/protocol/muc#user' code='coucou'/>
        ".parse().unwrap();
        let error = Status::try_from(elem).unwrap_err();
        let error = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(error.description(), "invalid digit found in string");
    }

    #[test]
    fn test_actor_required_attributes() {
        let elem: Element = "
            <actor xmlns='http://jabber.org/protocol/muc#user'/>
        ".parse().unwrap();
        let error = Actor::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Either 'jid' or 'nick' attribute is required.");
    }

    #[test]
    fn test_actor_required_attributes2() {
        let elem: Element = "
            <actor xmlns='http://jabber.org/protocol/muc#user'
                   jid='foo@bar/baz'
                   nick='baz'/>
        ".parse().unwrap();
        let error = Actor::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Either 'jid' or 'nick' attribute is required.");
    }

    #[test]
    fn test_actor_jid() {
        let elem: Element = "
            <actor xmlns='http://jabber.org/protocol/muc#user'
                   jid='foo@bar/baz'/>
        ".parse().unwrap();
        let actor = Actor::try_from(elem).unwrap();
        let jid = match actor {
            Actor::Jid(jid) => jid,
            _ => panic!(),
        };
        assert_eq!(jid, "foo@bar/baz".parse::<Jid>().unwrap());
    }

    #[test]
    fn test_actor_nick() {
        let elem: Element = "
            <actor xmlns='http://jabber.org/protocol/muc#user' nick='baz'/>
        ".parse().unwrap();
        let actor = Actor::try_from(elem).unwrap();
        let nick = match actor {
            Actor::Nick(nick) => nick,
            _ => panic!(),
        };
        assert_eq!(nick, "baz".to_owned());
    }

    #[test]
    fn test_continue_simple() {
        let elem: Element = "
            <continue xmlns='http://jabber.org/protocol/muc#user'/>
        ".parse().unwrap();
        Continue::try_from(elem).unwrap();
    }

    #[test]
    fn test_continue_thread_attribute() {
        let elem: Element = "
            <continue xmlns='http://jabber.org/protocol/muc#user'
                      thread='foo'/>
        ".parse().unwrap();
        let continue_ = Continue::try_from(elem).unwrap();
        assert_eq!(continue_, Continue { thread: Some("foo".to_owned()) });
    }

    #[test]
    fn test_continue_invalid() {
        let elem: Element = "
            <continue xmlns='http://jabber.org/protocol/muc#user'>
                <foobar/>
            </continue>
        ".parse().unwrap();
        let continue_ = Continue::try_from(elem).unwrap_err();
        let message = match continue_ {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in continue element.".to_owned());
    }

    #[test]
    fn test_reason_simple() {
        let elem: Element = "
            <reason xmlns='http://jabber.org/protocol/muc#user'>Reason</reason>"
        .parse().unwrap();
        let reason = Reason::try_from(elem).unwrap();
        assert_eq!(reason.0, "Reason".to_owned());
    }

    #[test]
    fn test_reason_invalid_attribute() {
        let elem: Element = "
            <reason xmlns='http://jabber.org/protocol/muc#user' foo='bar'/>
        ".parse().unwrap();
        let error = Reason::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in reason element.".to_owned());
    }

    #[test]
    fn test_reason_invalid() {
        let elem: Element = "
            <reason xmlns='http://jabber.org/protocol/muc#user'>
                <foobar/>
            </reason>
        ".parse().unwrap();
        let error = Reason::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in reason element.".to_owned());
    }

    #[test]
    fn test_item_invalid_attr(){
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  foo='bar'/>
        ".parse().unwrap();
        let error = Item::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in item element.".to_owned());
    }

    #[test]
    fn test_item_affiliation_role_attr(){
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  affiliation='member'
                  role='moderator'/>
        ".parse().unwrap();
        Item::try_from(elem).unwrap();
    }

    #[test]
    fn test_item_affiliation_role_invalid_attr(){
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  affiliation='member'/>
        ".parse().unwrap();
        let error = Item::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'role' missing.".to_owned());
    }

    #[test]
    fn test_item_nick_attr(){
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  affiliation='member'
                  role='moderator'
                  nick='foobar'/>
        ".parse().unwrap();
        let item = Item::try_from(elem).unwrap();
        match item {
            Item { nick, .. } => assert_eq!(nick, Some("foobar".to_owned())),
        }
    }

    #[test]
    fn test_item_affiliation_role_invalid_attr2(){
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  role='moderator'/>
        ".parse().unwrap();
        let error = Item::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'affiliation' missing.".to_owned());
    }

    #[test]
    fn test_item_role_actor_child(){
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  affiliation='member'
                  role='moderator'>
                <actor nick='foobar'/>
            </item>
        ".parse().unwrap();
        let item = Item::try_from(elem).unwrap();
        match item {
            Item { actor, .. } =>
                assert_eq!(actor, Some(Actor::Nick("foobar".to_owned()))),
        }
    }

    #[test]
    fn test_item_role_continue_child(){
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  affiliation='member'
                  role='moderator'>
                <continue thread='foobar'/>
            </item>
        ".parse().unwrap();
        let item = Item::try_from(elem).unwrap();
        let continue_1 = Continue { thread: Some("foobar".to_owned()) };
        match item {
            Item { continue_: Some(continue_2), .. } => assert_eq!(continue_2, continue_1),
            _ => panic!(),
        }
    }

    #[test]
    fn test_item_role_reason_child(){
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  affiliation='member'
                  role='moderator'>
                <reason>foobar</reason>
            </item>
        ".parse().unwrap();
        let item = Item::try_from(elem).unwrap();
        match item {
            Item { reason, .. } =>
                assert_eq!(reason, Some(Reason("foobar".to_owned()))),
        }
    }
}
