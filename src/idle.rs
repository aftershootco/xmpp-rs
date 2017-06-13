// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::convert::TryFrom;

use minidom::Element;
use chrono::{DateTime, FixedOffset};

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct Idle {
    pub since: DateTime<FixedOffset>,
}

impl TryFrom<Element> for Idle {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Idle, Error> {
        if !elem.is("idle", ns::IDLE) {
            return Err(Error::ParseError("This is not an idle element."));
        }
        for _ in elem.children() {
            return Err(Error::ParseError("Unknown child in idle element."));
        }
        let since = get_attr!(elem, "since", required, since, DateTime::parse_from_rfc3339(since)?);
        Ok(Idle { since: since })
    }
}

impl Into<Element> for Idle {
    fn into(self) -> Element {
        Element::builder("idle")
                .ns(ns::IDLE)
                .attr("since", self.since.to_rfc3339())
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_simple() {
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-21T20:19:55+01:00'/>".parse().unwrap();
        Idle::try_from(elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1'><coucou/></idle>".parse().unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in idle element.");
    }

    #[test]
    fn test_invalid_id() {
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1'/>".parse().unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'since' missing.");
    }

    #[test]
    fn test_invalid_date() {
        // There is no thirteenth month.
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-13-01T12:23:34Z'/>".parse().unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.description(), "input is out of range");

        // Timezone ≥24:00 aren’t allowed.
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-27T12:11:02+25:00'/>".parse().unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.description(), "input is out of range");

        // Timezone without the : separator aren’t allowed.
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-27T12:11:02+0100'/>".parse().unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.description(), "input contains invalid characters");

        // No seconds, error message could be improved.
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-27T12:11+01:00'/>".parse().unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.description(), "input contains invalid characters");

        // TODO: maybe we’ll want to support this one, as per XEP-0082 §4.
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='20170527T12:11:02+01:00'/>".parse().unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.description(), "input contains invalid characters");

        // No timezone.
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-27T12:11:02'/>".parse().unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.description(), "premature end of input");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-21T20:19:55+01:00'/>".parse().unwrap();
        let idle = Idle { since: DateTime::parse_from_rfc3339("2017-05-21T20:19:55+01:00").unwrap() };
        let elem2 = idle.into();
        assert_eq!(elem, elem2);
    }
}