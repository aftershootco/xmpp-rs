// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use try_from::TryFrom;

use minidom::Element;
use jid::Jid;

use error::Error;

use iq::{IqGetPayload, IqSetPayload, IqResultPayload};
use data_forms::DataForm;
use rsm::Set;
use forwarding::Forwarded;
use pubsub::NodeName;

use ns;

generate_id!(
    /// An identifier matching a result message to the query requesting it.
    QueryId
);

generate_element!(
    Query, "query", MAM,
    attributes: [
        queryid: Option<QueryId> = "queryid" => optional,
        node: Option<NodeName> = "node" => optional
    ],
    children: [
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm,
        set: Option<Set> = ("set", RSM) => Set
    ]
);

impl IqGetPayload for Query {}
impl IqSetPayload for Query {}
impl IqResultPayload for Query {}

generate_element!(
    Result_, "result", MAM,
    attributes: [
        id: String = "id" => required,
        queryid: Option<QueryId> = "queryid" => optional,
    ],
    children: [
        forwarded: Required<Forwarded> = ("forwarded", FORWARD) => Forwarded
    ]
);

generate_attribute!(
    Complete, "complete", bool
);

generate_element!(
    Fin, "fin", MAM,
    attributes: [
        complete: Complete = "complete" => default
    ],
    children: [
        set: Required<Set> = ("set", RSM) => Set
    ]
);

impl IqResultPayload for Fin {}

generate_attribute!(DefaultPrefs, "default", {
    Always => "always",
    Never => "never",
    Roster => "roster",
});

#[derive(Debug, Clone)]
pub struct Prefs {
    pub default_: DefaultPrefs,
    pub always: Vec<Jid>,
    pub never: Vec<Jid>,
}

impl IqGetPayload for Prefs {}
impl IqSetPayload for Prefs {}
impl IqResultPayload for Prefs {}

impl TryFrom<Element> for Prefs {
    type Err = Error;

    fn try_from(elem: Element) -> Result<Prefs, Error> {
        check_self!(elem, "prefs", MAM);
        check_no_unknown_attributes!(elem, "prefs", ["default"]);
        let mut always = vec!();
        let mut never = vec!();
        for child in elem.children() {
            if child.is("always", ns::MAM) {
                for jid_elem in child.children() {
                    if !jid_elem.is("jid", ns::MAM) {
                        return Err(Error::ParseError("Invalid jid element in always."));
                    }
                    always.push(jid_elem.text().parse()?);
                }
            } else if child.is("never", ns::MAM) {
                for jid_elem in child.children() {
                    if !jid_elem.is("jid", ns::MAM) {
                        return Err(Error::ParseError("Invalid jid element in never."));
                    }
                    never.push(jid_elem.text().parse()?);
                }
            } else {
                return Err(Error::ParseError("Unknown child in prefs element."));
            }
        }
        let default_ = get_attr!(elem, "default", required);
        Ok(Prefs { default_, always, never })
    }
}

fn serialise_jid_list(name: &str, jids: Vec<Jid>) -> Option<Element> {
    if jids.is_empty() {
        None
    } else {
        Some(Element::builder(name)
                     .ns(ns::MAM)
                     .append(jids.into_iter()
                                 .map(|jid| Element::builder("jid")
                                                    .ns(ns::MAM)
                                                    .append(jid)
                                                    .build())
                                 .collect::<Vec<_>>())
                     .build())
    }
}

impl From<Prefs> for Element {
    fn from(prefs: Prefs) -> Element {
        Element::builder("prefs")
                .ns(ns::MAM)
                .attr("default", prefs.default_)
                .append(serialise_jid_list("always", prefs.always))
                .append(serialise_jid_list("never", prefs.never))
                .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_query() {
        let elem: Element = "<query xmlns='urn:xmpp:mam:2'/>".parse().unwrap();
        Query::try_from(elem).unwrap();
    }

    #[test]
    fn test_result() {
        #[cfg(not(feature = "component"))]
        let elem: Element = r#"
<result xmlns='urn:xmpp:mam:2' queryid='f27' id='28482-98726-73623'>
  <forwarded xmlns='urn:xmpp:forward:0'>
    <delay xmlns='urn:xmpp:delay' stamp='2010-07-10T23:08:25Z'/>
    <message xmlns='jabber:client' from="witch@shakespeare.lit" to="macbeth@shakespeare.lit">
      <body>Hail to thee</body>
    </message>
  </forwarded>
</result>
"#.parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = r#"
<result xmlns='urn:xmpp:mam:2' queryid='f27' id='28482-98726-73623'>
  <forwarded xmlns='urn:xmpp:forward:0'>
    <delay xmlns='urn:xmpp:delay' stamp='2010-07-10T23:08:25Z'/>
    <message xmlns='jabber:component:accept' from="witch@shakespeare.lit" to="macbeth@shakespeare.lit">
      <body>Hail to thee</body>
    </message>
  </forwarded>
</result>
"#.parse().unwrap();
        Result_::try_from(elem).unwrap();
    }

    #[test]
    fn test_fin() {
        let elem: Element = r#"
<fin xmlns='urn:xmpp:mam:2'>
  <set xmlns='http://jabber.org/protocol/rsm'>
    <first index='0'>28482-98726-73623</first>
    <last>09af3-cc343-b409f</last>
  </set>
</fin>
"#.parse().unwrap();
        Fin::try_from(elem).unwrap();
    }

    #[test]
    fn test_query_x() {
        let elem: Element = r#"
<query xmlns='urn:xmpp:mam:2'>
  <x xmlns='jabber:x:data' type='submit'>
    <field var='FORM_TYPE' type='hidden'>
      <value>urn:xmpp:mam:2</value>
    </field>
    <field var='with'>
      <value>juliet@capulet.lit</value>
    </field>
  </x>
</query>
"#.parse().unwrap();
        Query::try_from(elem).unwrap();
    }

    #[test]
    fn test_query_x_set() {
        let elem: Element = r#"
<query xmlns='urn:xmpp:mam:2'>
  <x xmlns='jabber:x:data' type='submit'>
    <field var='FORM_TYPE' type='hidden'>
      <value>urn:xmpp:mam:2</value>
    </field>
    <field var='start'>
      <value>2010-08-07T00:00:00Z</value>
    </field>
  </x>
  <set xmlns='http://jabber.org/protocol/rsm'>
    <max>10</max>
  </set>
</query>
"#.parse().unwrap();
        Query::try_from(elem).unwrap();
    }

    #[test]
    fn test_prefs_get() {
        let elem: Element = "<prefs xmlns='urn:xmpp:mam:2' default='always'/>".parse().unwrap();
        let prefs = Prefs::try_from(elem).unwrap();
        assert_eq!(prefs.always, vec!());
        assert_eq!(prefs.never, vec!());

        let elem: Element = r#"
<prefs xmlns='urn:xmpp:mam:2' default='roster'>
  <always/>
  <never/>
</prefs>
"#.parse().unwrap();
        let prefs = Prefs::try_from(elem).unwrap();
        assert_eq!(prefs.always, vec!());
        assert_eq!(prefs.never, vec!());
    }

    #[test]
    fn test_prefs_result() {
        let elem: Element = r#"
<prefs xmlns='urn:xmpp:mam:2' default='roster'>
  <always>
    <jid>romeo@montague.lit</jid>
  </always>
  <never>
    <jid>montague@montague.lit</jid>
  </never>
</prefs>
"#.parse().unwrap();
        let prefs = Prefs::try_from(elem).unwrap();
        assert_eq!(prefs.always, vec!(Jid::from_str("romeo@montague.lit").unwrap()));
        assert_eq!(prefs.never, vec!(Jid::from_str("montague@montague.lit").unwrap()));

        let elem2 = Element::from(prefs.clone());
        println!("{:?}", elem2);
        let prefs2 = Prefs::try_from(elem2).unwrap();
        assert_eq!(prefs.default_, prefs2.default_);
        assert_eq!(prefs.always, prefs2.always);
        assert_eq!(prefs.never, prefs2.never);
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<query xmlns='urn:xmpp:mam:2'><coucou/></query>".parse().unwrap();
        let error = Query::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in query element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<query xmlns='urn:xmpp:mam:2'/>".parse().unwrap();
        let replace = Query { queryid: None, node: None, form: None, set: None };
        let elem2 = replace.into();
        assert_eq!(elem, elem2);
    }
}
